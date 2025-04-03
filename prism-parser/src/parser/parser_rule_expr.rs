use crate::core::adaptive::{BlockState, GrammarState, RuleId};
use crate::core::arc_ref::BorrowedArcSlice;
use crate::core::context::{PR, ParserContext};
use crate::core::input::Input;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use crate::grammar::rule_expr::RuleExpr;
use crate::parsable::guid::Guid;
use crate::parsable::parsed::{ArcExt, Parsed};
use crate::parsable::void::Void;
use crate::parser::VarMap;
use crate::parser::parsed_list::ParsedList;
use crate::parser::placeholder_store::ParsedPlaceholder;
use crate::parser::rule_closure::RuleClosure;
use std::collections::HashMap;
use std::sync::Arc;

impl<Db, E: ParseError<L = ErrorLabel>> ParserState<Db, E> {
    pub fn parse_expr(
        &mut self,
        expr: &RuleExpr,
        rules: &GrammarState,
        blocks: BorrowedArcSlice<Arc<BlockState>>,
        rule_args: &VarMap,
        vars: &VarMap,
        pos: Pos,
        context: ParserContext,
        penv: &mut Db,
        eval_ctx: &Parsed,
        eval_ctxs: &mut HashMap<String, (Parsed, ParsedPlaceholder)>,
    ) -> PResult<PR, E> {
        match expr {
            RuleExpr::RunVar { rule, args } => {
                let mut arg_values = Vec::new();
                for arg in &**args {
                    arg_values.push(if let RuleExpr::RunVar { rule: r, args } = &**arg {
                        let r_str = r.as_str();
                        if args.is_empty() && !["#this", "#next"].contains(&r_str) {
                            vars.get(r).unwrap().clone()
                        } else {
                            Arc::new(RuleClosure {
                                expr: arg.clone(),
                                blocks: blocks.to_cloned(),
                                rule_args: rule_args.clone(),
                                vars: vars.clone(),
                            })
                            .to_parsed()
                        }
                    // Edge-case, optimization $v expressions
                    } else if let RuleExpr::Action(seq, action) = &**arg
                        && let RuleExpr::Sequence(seqs) = &**seq
                        && seqs.is_empty()
                    {
                        self.apply_action(action, pos.span_to(pos), vars, penv)
                    } else {
                        Arc::new(RuleClosure {
                            expr: arg.clone(),
                            blocks: blocks.to_cloned(),
                            rule_args: rule_args.clone(),
                            vars: vars.clone(),
                        })
                        .to_parsed()
                    })
                }

                // Handle #this and #next logic
                let rule_str = rule.as_str();
                if rule_str == "#this" || rule_str == "#next" {
                    let blocks = match rule_str {
                        "#this" => blocks,
                        "#next" => blocks.slice(1..),
                        _ => unreachable!(),
                    };
                    let arg_values = if arg_values.is_empty() {
                        rule_args
                    } else {
                        assert_eq!(arg_values.len(), rule_args.len());
                        &VarMap::from_iter(
                            rule_args
                                .iter()
                                .collect::<Vec<_>>()
                                .into_iter()
                                .rev()
                                .zip(arg_values)
                                .map(|((n, _), v)| (n.clone(), v)),
                        )
                    };
                    return self
                        .parse_rule_block(rules, blocks, arg_values, pos, context, penv, eval_ctx)
                        .map(PR::with_rtrn);
                }

                // Figure out which rule the variable `rule` refers to
                let Some(rule) = vars.get(rule) else {
                    panic!("Tried to run variable `{rule_str}` as a rule, but it was not defined.");
                };
                if let Some(rule) = rule.try_value_ref::<RuleId>() {
                    self.parse_rule(rules, *rule, &arg_values, pos, context, penv, eval_ctx)
                        .map(PR::with_rtrn)
                } else if let Some(closure) = rule.try_value_ref::<RuleClosure>() {
                    assert_eq!(arg_values.len(), 0);
                    self.parse_expr(
                        &closure.expr,
                        rules,
                        closure.blocks.to_borrowed(),
                        &closure.rule_args,
                        &closure.vars,
                        pos,
                        context,
                        penv,
                        eval_ctx,
                        &mut HashMap::new(),
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
                .map(|(span, _)| {
                    PR::with_rtrn(Arc::new(Input::from_span(span, &self.input)).to_parsed())
                }),
            RuleExpr::Literal(literal) => self.parse_with_layout(
                rules,
                vars,
                |state, start_pos, _penv| {
                    let mut res = PResult::new_empty((), start_pos);
                    for char in literal.as_str().chars() {
                        let new_res = state.parse_char(|c| *c == char, res.end_pos());
                        res = res.merge_seq(new_res).map(|_| ());
                    }
                    let span = start_pos.span_to(res.end_pos());
                    let mut res = res.map(|_| {
                        PR::with_rtrn(Arc::new(Input::from_span(span, &state.input)).to_parsed())
                    });
                    res.add_label_implicit(ErrorLabel::Literal(span, literal.to_string()));
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
                        self.parse_expr(
                            expr,
                            rules,
                            blocks,
                            rule_args,
                            vars,
                            pos,
                            context,
                            penv,
                            eval_ctx,
                            &mut HashMap::new(),
                        )
                    } else {
                        self.parse_expr(
                            delim,
                            rules,
                            blocks,
                            rule_args,
                            vars,
                            pos,
                            context,
                            penv,
                            eval_ctx,
                            &mut HashMap::new(),
                        )
                        .merge_seq_chain(|pos| {
                            self.parse_expr(
                                expr,
                                rules,
                                blocks,
                                rule_args,
                                vars,
                                pos,
                                context,
                                penv,
                                eval_ctx,
                                &mut HashMap::new(),
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
                        e.add_label_explicit(ErrorLabel::Debug(
                            pos.span_to(pos),
                            "INFLOOP".to_string(),
                        ));
                        return PResult::new_err(e, pos);
                    }
                }

                res.map(|rtrn| {
                    rtrn.iter().rfold(ParsedList::default(), |rest, next| {
                        rest.insert((), next.rtrn.clone())
                    })
                })
                .map(|ar| PR::with_rtrn(Arc::new(ar).to_parsed()))
            }
            RuleExpr::Sequence(subs) => {
                let mut res = PResult::new_empty(VarMap::default(), pos);
                let mut res_vars: VarMap = vars.clone();
                for sub in &**subs {
                    res = res
                        .merge_seq_chain(|pos| {
                            self.parse_expr(
                                sub, rules, blocks, rule_args, &res_vars, pos, context, penv,
                                eval_ctx, eval_ctxs,
                            )
                        })
                        .map(|(l, r)| l.extend(r.free.iter_cloned()));
                    match &res.ok_ref() {
                        None => break,
                        Some(o) => {
                            res_vars = res_vars.extend(o.iter_cloned());
                        }
                    }
                }
                res.map(|map| PR {
                    free: map,
                    rtrn: Arc::new(Void).to_parsed(),
                })
            }
            RuleExpr::Choice(subs) => {
                let mut res: PResult<PR, E> = PResult::PErr {
                    err: E::new(pos),
                    end: pos,
                };
                for sub in &**subs {
                    res = res.merge_choice_chain(|| {
                        self.parse_expr(
                            sub,
                            rules,
                            blocks,
                            rule_args,
                            vars,
                            pos,
                            context,
                            penv,
                            eval_ctx,
                            &mut HashMap::new(),
                        )
                    });
                    if res.is_ok() {
                        break;
                    }
                }
                res
            }
            RuleExpr::NameBind(name, sub) => {
                let (eval_ctx, placeholder) =
                    if let Some((eval_ctx, placeholder)) = eval_ctxs.get(name.as_str()) {
                        (eval_ctx, Some(*placeholder))
                    } else {
                        (eval_ctx, None)
                    };

                let res = self.parse_expr(
                    sub,
                    rules,
                    blocks,
                    rule_args,
                    vars,
                    pos,
                    context,
                    penv,
                    eval_ctx,
                    &mut HashMap::new(),
                );
                res.map_with_span(|res, span| {
                    if let Some(placeholder) = placeholder {
                        self.placeholders
                            .place_into_empty(placeholder, res.rtrn.clone(), penv);
                    }

                    PR {
                        free: res.free.insert(name.clone(), res.rtrn.clone()),
                        rtrn: Arc::new(Void).to_parsed(),
                    }
                })
            }
            RuleExpr::Action(sub, action) => {
                let mut eval_ctxs = HashMap::new();
                let root_placeholder = self.placeholders.push_empty();
                self.pre_apply_action(action, penv, root_placeholder, eval_ctx, &mut eval_ctxs);

                let res = self.parse_expr(
                    sub,
                    rules,
                    blocks,
                    rule_args,
                    vars,
                    pos,
                    context,
                    penv,
                    eval_ctx,
                    &mut eval_ctxs,
                );

                //TODO
                // res.map_with_span(|res, span| {
                //     let parsed = self.placeholders.get(root_placeholder).unwrap();
                //     PR::with_rtrn(
                //         parsed.clone()
                //     )
                // })

                res.map_with_span(|res, span| {
                    PR::with_rtrn(self.apply_action(
                        action,
                        span,
                        &vars.extend(res.free.iter_cloned()),
                        penv,
                    ))
                })
            }
            RuleExpr::SliceInput(sub) => {
                let res = self.parse_expr(
                    sub,
                    rules,
                    blocks,
                    rule_args,
                    vars,
                    pos,
                    context,
                    penv,
                    eval_ctx,
                    &mut HashMap::new(),
                );
                res.map_with_span(|_, span| {
                    PR::with_rtrn(Arc::new(Input::from_span(span, &self.input)).to_parsed())
                })
            }
            RuleExpr::PosLookahead(sub) => self
                .parse_expr(
                    sub,
                    rules,
                    blocks,
                    rule_args,
                    vars,
                    pos,
                    context,
                    penv,
                    eval_ctx,
                    &mut HashMap::new(),
                )
                .positive_lookahead(pos),
            RuleExpr::NegLookahead(sub) => self
                .parse_expr(
                    sub,
                    rules,
                    blocks,
                    rule_args,
                    vars,
                    pos,
                    context,
                    penv,
                    eval_ctx,
                    &mut HashMap::new(),
                )
                .negative_lookahead(pos)
                .map(|()| PR::with_rtrn(Arc::new(Void).to_parsed())),
            RuleExpr::AtAdapt {
                ns,
                name: grammar,
                expr: body,
            } => {
                let ns = ns.as_str();

                let ns = self
                    .parsables
                    .get(ns)
                    .unwrap_or_else(|| panic!("Namespace '{ns}' exists"));
                let grammar = vars.get(grammar).unwrap();
                let grammar =
                    (ns.eval_to_grammar)(grammar, eval_ctx, &self.placeholders, &self.input, penv);

                // Create new grammarstate
                //TODO performance: we shoud cache grammar states
                //TODO this should not use `vars`, but instead the global scope in which this rule is defined
                let (rules, _) = match rules.adapt_with(&grammar, vars, Some(pos), &self.input) {
                    Ok(rules) => rules,
                    Err(_) => {
                        let mut e = E::new(pos);
                        e.add_label_implicit(ErrorLabel::Explicit(
                            pos.span_to(pos),
                            "language grammar to be correct, but adaptation created cycle in block order.".to_string(),
                        ));
                        return PResult::new_err(e, pos);
                    }
                };
                let rules: Arc<GrammarState> = Arc::new(rules);

                let mut res = self.parse_expr(
                    body, &rules, blocks, rule_args, vars, pos, context, penv, eval_ctx, eval_ctxs,
                );
                res.add_label_implicit(ErrorLabel::Debug(
                    pos.span_to(pos),
                    "adaptation".to_string(),
                ));
                res
            }
            RuleExpr::Guid => {
                let guid = Guid(self.guid_counter);
                self.guid_counter += 1;

                PResult::new_empty(PR::with_rtrn(Arc::new(guid).to_parsed()), pos)
            }
        }
    }
}
