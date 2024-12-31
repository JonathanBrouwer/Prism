use crate::core::adaptive::{BlockState, GrammarState, RuleId};
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
use crate::parsable::parsed::Parsed;
use crate::parsable::void::Void;
use crate::parsable::ParseResult;
use crate::parser::parsed_list::ParsedList;
use crate::parser::rule_closure::RuleClosure;
use crate::parser::var_map::VarMap;

impl<'arn, 'grm: 'arn, Env, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, Env, E> {
    pub fn parse_expr(
        &mut self,
        rules: &'arn GrammarState<'arn, 'grm>,
        blocks: &'arn [BlockState<'arn, 'grm>],
        rule_args: VarMap<'arn, 'grm>,
        expr: &'arn RuleExpr<'arn, 'grm>,
        vars: VarMap<'arn, 'grm>,
        pos: Pos,
        context: ParserContext,
        penv: &mut Env,
    ) -> PResult<PR<'arn, 'grm>, E> {
        match expr {
            RuleExpr::RunVar { rule, args } => {
                // Figure out which rule the variable `rule` refers to
                let Some(rule) = vars.get(rule) else {
                    panic!("Tried to run variable `{rule}` as a rule, but it was not defined.");
                };

                let mut arg_values = Vec::new();
                for arg in *args {
                    arg_values.push(if let RuleExpr::RunVar { rule: r, args } = arg {
                        if args.is_empty() {
                            *vars.get(r).unwrap()
                        } else {
                            self.alloc
                                .alloc(RuleClosure {
                                    expr: arg,
                                    blocks,
                                    rule_args,
                                    vars,
                                })
                                .to_parsed()
                        }
                    } else if let RuleExpr::Action(RuleExpr::Sequence([]), action) = arg {
                        match self.apply_action(action, pos.span_to(pos), vars, penv) {
                            Ok(v) => v,
                            Err(e) => return PResult::new_err(e, pos),
                        }
                    } else {
                        self.alloc
                            .alloc(RuleClosure {
                                expr: arg,
                                blocks,
                                rule_args,
                                vars,
                            })
                            .to_parsed()
                    })
                }

                if let Some(rule) = rule.try_into_value::<RuleId>() {
                    self.parse_rule(rules, *rule, &arg_values, pos, context, penv)
                        .map(PR::with_rtrn)
                } else if let Some(closure) = rule.try_into_value::<RuleClosure>() {
                    assert_eq!(arg_values.len(), 0);
                    self.parse_expr(
                        rules,
                        closure.blocks,
                        closure.rule_args,
                        closure.expr,
                        closure.vars,
                        pos,
                        context,
                        penv,
                    )
                } else {
                    panic!("Tried to run a rule of value type: {}", rule.name)
                }
            }
            RuleExpr::CharClass(cc) => self
                .parse_with_layout(
                    rules,
                    vars,
                    |state, pos, _penv| state.parse_char(|c| cc.contains(*c), pos),
                    pos,
                    context,
                    penv,
                )
                .map(|(span, _)| PR::with_rtrn(self.alloc.alloc(Input::Value(span)).to_parsed())),
            RuleExpr::Literal(literal) => self.parse_with_layout(
                rules,
                vars,
                |state, start_pos, _penv| {
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
                penv,
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
                        self.parse_expr(rules, blocks, rule_args, expr, vars, pos, context, penv)
                    } else {
                        self.parse_expr(rules, blocks, rule_args, delim, vars, pos, context, penv)
                            .merge_seq_chain(|pos| {
                                self.parse_expr(
                                    rules, blocks, rule_args, expr, vars, pos, context, penv,
                                )
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
                        let mut e = E::new(pos);
                        e.add_label_explicit(ErrorLabel::Debug(pos.span_to(pos), "INFLOOP"));
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
                            self.parse_expr(
                                rules, blocks, rule_args, sub, res_vars, pos, context, penv,
                            )
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
                let mut res: PResult<PR, E> = PResult::PErr(E::new(pos), pos);
                for sub in *subs {
                    res = res.merge_choice_chain(|| {
                        self.parse_expr(rules, blocks, rule_args, sub, vars, pos, context, penv)
                    });
                    if res.is_ok() {
                        break;
                    }
                }
                res
            }
            RuleExpr::NameBind(name, sub) => {
                let res = self.parse_expr(rules, blocks, rule_args, sub, vars, pos, context, penv);
                res.map(|res| PR {
                    free: res.free.insert(name, res.rtrn, self.alloc),
                    rtrn: Void.to_parsed(),
                })
            }
            RuleExpr::Action(sub, action) => {
                let res = self.parse_expr(rules, blocks, rule_args, sub, vars, pos, context, penv);
                res.merge_seq_chain2(|pos, span, res| {
                    match self.apply_action(
                        action,
                        span,
                        res.free.extend(vars.iter_cloned(), self.alloc),
                        penv,
                    ) {
                        Ok(rtrn) => PResult::new_empty(
                            PR {
                                free: res.free,
                                rtrn,
                            },
                            pos,
                        ),
                        Err(e) => return PResult::PErr(e, pos),
                    }
                })
            }
            RuleExpr::SliceInput(sub) => {
                let res = self.parse_expr(rules, blocks, rule_args, sub, vars, pos, context, penv);
                res.map_with_span(|_, span| {
                    PR::with_rtrn(self.alloc.alloc(Input::Value(span)).to_parsed())
                })
            }
            RuleExpr::This => self
                .parse_rule_block(rules, blocks, rule_args, pos, context, penv)
                .map(PR::with_rtrn),
            RuleExpr::Next => self
                .parse_rule_block(rules, &blocks[1..], rule_args, pos, context, penv)
                .map(PR::with_rtrn),
            RuleExpr::PosLookahead(sub) => self
                .parse_expr(rules, blocks, rule_args, sub, vars, pos, context, penv)
                .positive_lookahead(pos),
            RuleExpr::NegLookahead(sub) => self
                .parse_expr(rules, blocks, rule_args, sub, vars, pos, context, penv)
                .negative_lookahead(pos)
                .map(|()| PR::with_rtrn(Void.to_parsed())),
            RuleExpr::AtAdapt(ga, body) => {
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
                        let mut e = E::new(pos);
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

                let mut res =
                    self.parse_expr(rules, blocks, rule_args, body, vars, pos, context, penv);
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
