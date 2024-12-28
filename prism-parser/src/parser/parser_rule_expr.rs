use crate::core::adaptive::{GrammarState, RuleId};
use crate::core::context::{ParserContext, PR};
use crate::core::input::Input;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::grammar_file::GrammarFile;
use crate::grammar::rule_expr::RuleExpr;
use crate::parsable::guid::Guid;
use crate::parsable::void::Void;
use crate::parsable::Parsable;
use crate::parser::parsed_list::ParsedList;
use crate::parser::var_map::{BlockCtx, CapturedExpr, VarMap, VarMapValue};

impl<'arn, 'grm, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    pub fn parse_expr(
        &mut self,
        rules: &'arn GrammarState<'arn, 'grm>,
        block_ctx: BlockCtx<'arn, 'grm>,
        expr: &'arn RuleExpr<'arn, 'grm>,
        vars: VarMap<'arn, 'grm>,
        pos: Pos,
        context: ParserContext,
    ) -> PResult<PR<'arn, 'grm>, E> {
        match expr {
            RuleExpr::RunVar(mut rule_str, args) => {
                // Figure out which rule the variable `rule` refers to
                let Some(rule) = vars.get(rule_str) else {
                    panic!("Tried to run variable `{rule_str}` as a rule, but it was not defined.");
                };
                let rule = *rule.into_value::<RuleId>();

                let mut res = PResult::new_empty(Vec::new(), pos);
                for arg in *args {
                    if let RuleExpr::RunVar(r, args) = arg {
                        assert_eq!(args.len(), 0);
                        res = res.map(|mut v| {
                            v.push(*vars.get(r).unwrap());
                            v
                        });
                    } else {
                        res = res
                            .merge_seq(self.parse_expr(rules, block_ctx, arg, vars, pos, context))
                            .map(|(mut v, r)| {
                                v.push(r.rtrn);
                                v
                            });
                    }
                }

                res.merge_seq_chain2(|pos, args| {
                    self.parse_rule(rules, rule, &args, pos, context)
                        .map(PR::with_rtrn)
                })
            }
            RuleExpr::CharClass(cc) => self
                .parse_with_layout(
                    rules,
                    vars,
                    |state, pos| state.parse_char(|c| cc.contains(*c), pos),
                    pos,
                    context,
                )
                .map(|(span, _)| PR::with_rtrn(self.alloc.alloc(Input::Value(span)).to_parsed())),
            RuleExpr::Literal(literal) => self.parse_with_layout(
                rules,
                vars,
                |state, start_pos| {
                    let mut res = PResult::new_empty((), start_pos);
                    for char in literal.chars() {
                        let new_res = state.parse_char(|c| *c == char, res.end_pos());
                        res = res.merge_seq(new_res).map(|_| ());
                    }
                    let span = start_pos.span_to(res.end_pos());
                    let mut res = res
                        .map(|_| PR::with_rtrn(state.alloc.alloc(Input::Value(span)).to_parsed()));
                    res.add_label_implicit(ErrorLabel::Literal(span, *literal));
                    res
                },
                pos,
                context,
            ),
            RuleExpr::Repeat {
                expr,
                min,
                max,
                delim,
            } => {
                let mut res: PResult<Vec<_>, E> = PResult::new_empty(vec![], pos);

                for i in 0..max.unwrap_or(u64::MAX) {
                    let pos = res.end_pos();
                    let part = if i == 0 {
                        self.parse_expr(rules, block_ctx, expr, vars, pos, context)
                    } else {
                        self.parse_expr(rules, block_ctx, delim, vars, pos, context)
                            .merge_seq_chain(|pos| {
                                self.parse_expr(rules, block_ctx, expr, vars, pos, context)
                            })
                            .map(|x| x.1)
                    };
                    let should_continue = part.is_ok();

                    if i < *min {
                        res = res.merge_seq(part).map(|(mut vec, item)| {
                            vec.push(item);
                            vec
                        });
                    } else {
                        res = res.merge_seq_opt(part).map(|(mut vec, item)| {
                            if let Some(item) = item {
                                vec.push(item);
                            }
                            vec
                        });
                    };

                    if !should_continue {
                        break;
                    };

                    // If the result is OK and the last pos has not changed, we got into an infinite loop
                    // We break out with an infinite loop error
                    // The i != 0 check is to make sure to take the delim into account
                    if i != 0 && res.end_pos() <= pos {
                        let span = pos.span_to(pos);
                        let mut e = E::new(span);
                        e.add_label_explicit(ErrorLabel::Debug(span, "INFLOOP"));
                        return PResult::new_err(e, pos);
                    }
                }

                res.map(|rtrn| {
                    rtrn.iter().rfold(ParsedList::new_empty(), |rest, next| {
                        rest.cons(next.rtrn, self.alloc)
                    })
                })
                .map(|ar| PR::with_rtrn(self.alloc.alloc(ar).to_parsed()))
            }
            RuleExpr::Sequence(subs) => {
                let mut res = PResult::new_empty(VarMap::default(), pos);
                //TODO can we do better than tracking res_vars by cloning?
                let mut res_vars: VarMap = vars;
                for sub in *subs {
                    res = res
                        .merge_seq_chain(|pos| {
                            self.parse_expr(rules, block_ctx, sub, res_vars, pos, context)
                        })
                        .map(|(l, r)| l.extend(r.free.iter_cloned(), self.alloc));
                    match &res.ok_ref() {
                        None => break,
                        Some(o) => {
                            res_vars = res_vars.extend(o.iter_cloned(), self.alloc);
                        }
                    }
                }
                res.map(|map| PR {
                    free: map,
                    rtrn: Void.to_parsed(),
                })
            }
            RuleExpr::Choice(subs) => {
                let mut res: PResult<PR, E> = PResult::PErr(E::new(pos.span_to(pos)), pos);
                for sub in *subs {
                    res = res.merge_choice_chain(|| {
                        self.parse_expr(rules, block_ctx, sub, vars, pos, context)
                    });
                    if res.is_ok() {
                        break;
                    }
                }
                res
            }
            RuleExpr::NameBind(name, sub) => {
                let res = self.parse_expr(rules, block_ctx, sub, vars, pos, context);
                res.map(|res| PR {
                    free: res.free.insert(name, res.rtrn, self.alloc),
                    rtrn: Void.to_parsed(),
                })
            }
            RuleExpr::Action(sub, action) => {
                let res = self.parse_expr(rules, block_ctx, sub, vars, pos, context);
                res.map_with_span(|res, span| {
                    let rtrn = self.apply_action(
                        action,
                        span,
                        res.free.extend(vars.iter_cloned(), self.alloc),
                    );

                    PR {
                        free: res.free,
                        rtrn,
                    }
                })
            }
            RuleExpr::SliceInput(sub) => {
                let res = self.parse_expr(rules, block_ctx, sub, vars, pos, context);
                res.map_with_span(|_, span| {
                    PR::with_rtrn(self.alloc.alloc(Input::Value(span)).to_parsed())
                })
            }
            RuleExpr::This => self
                .parse_rule_block(rules, block_ctx, pos, context)
                .map(PR::with_rtrn),
            RuleExpr::Next => self
                .parse_rule_block(rules, (&block_ctx.0[1..], block_ctx.1), pos, context)
                .map(PR::with_rtrn),
            RuleExpr::PosLookahead(sub) => self
                .parse_expr(rules, block_ctx, sub, vars, pos, context)
                .positive_lookahead(pos),
            RuleExpr::NegLookahead(sub) => self
                .parse_expr(rules, block_ctx, sub, vars, pos, context)
                .negative_lookahead(pos)
                .map(|()| PR::with_rtrn(Void.to_parsed())),
            RuleExpr::AtAdapt(ga, adapt_rule) => {
                // First, get the grammar actionresult
                let grammar = if let Some(ar) = vars.get(ga) {
                    *ar
                } else {
                    panic!("Name '{ga}' not in context")
                };

                // Parse it into a grammar
                let grammar = grammar.into_value::<GrammarFile>();

                // Create new grammarstate
                //TODO performance: we shoud cache grammar states
                let (rules, _) = match rules.adapt_with(grammar, vars, Some(pos), self.alloc) {
                    Ok(rules) => rules,
                    Err(_) => {
                        let mut e = E::new(pos.span_to(pos));
                        e.add_label_implicit(ErrorLabel::Explicit(
                            pos.span_to(pos),
                            EscapedString::from_escaped(
                                "language grammar to be correct, but adaptation created cycle in block order.",
                            ),
                        ));
                        return PResult::new_err(e, pos);
                    }
                };
                let rules: &'arn GrammarState = self.alloc.alloc(rules);

                let rule = *vars
                    .get(adapt_rule)
                    .or_else(|| vars.get(adapt_rule))
                    .unwrap()
                    .into_value::<RuleId>();

                // Parse body
                let mut res = self
                    .parse_rule(rules, rule, &[], pos, context)
                    .map(PR::with_rtrn);
                res.add_label_implicit(ErrorLabel::Debug(pos.span_to(pos), "adaptation"));
                res
            }
            RuleExpr::Guid => {
                let guid = Guid(self.guid_counter);
                self.guid_counter += 1;

                PResult::new_empty(PR::with_rtrn(self.alloc.alloc(guid).to_parsed()), pos)
            }
        }
    }
}
