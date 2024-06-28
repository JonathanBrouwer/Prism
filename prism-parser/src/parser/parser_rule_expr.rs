use crate::core::adaptive::{BlockState, GrammarState};
use crate::core::context::{ParserContext, PR};
use crate::core::cow::Cow;
use crate::core::parser::{map_parser, Parser};
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::primitives::{negative_lookahead, positive_lookahead, repeat_delim, single};
use crate::core::recovery::recovery_point;
use crate::core::span::Span;
use crate::core::state::PState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::from_action_result::parse_grammarfile;
use crate::grammar::{GrammarFile, RuleArg, RuleExpr};
use crate::parser::parser_layout::parser_with_layout;
use crate::parser::parser_rule::parser_rule;
use crate::parser::parser_rule_body::parser_body_cache_recurse;
use crate::parser::var_map::{BlockCtx, CapturedExpr, VarMap, VarMapValue};
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::apply_action::apply_action;
use crate::rule_action::RuleAction;
use by_address::ByAddress;

// pub fn parser_expr_rule<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
//     rules: &'arn GrammarState<'arn, 'grm>,
//     blocks: &'arn [BlockState<'arn, 'grm>],
// 
//     rule: &str,
//     expr_args: &[RuleArg<'grm, RuleAction<'arn, 'grm>>],
// 
//     // TODO merge this with `blocks`
//     rule_args: VarMap<'arn, 'grm>,
//     vars: VarMap<'arn, 'grm>,
// ) -> impl Parser<'arn, 'grm, PR<'arn, 'grm>, E> + 'a {
//     move |pos: Pos,
//           state: &mut PState<'arn, 'grm, E>,
//           context: ParserContext|
//           -> PResult<PR<'arn, 'grm>, E> {
//         // Figure out which rule the variable `rule` refers to
//         let Some(rule) = vars.get(rule) else {
//             panic!("Tried to run variable `{rule}` as a rule, but it was not defined.");
//         };
// 
//         let mut presult = PResult::new_ok((), pos, pos);
//         let mut result_args: Vec<VarMapValue> = vec![];
//         for arg in args {
//             let arg = match arg {
//                 RuleArg::ByValue(arg) => {
//                     let arg_result = presult.merge_seq_parser(
//                         &parser_expr(rules, blocks, arg, rule_args, vars),
//                         state,
//                         context,
//                     );
//                     let PResult::POk(arg, _, _, _) = arg_result.clone() else {
//                         return arg_result.map(|(_, arg)| arg);
//                     };
//                     presult = arg_result.map(|_| ());
//                     VarMapValue::Value(arg.1.rtrn)
//                 }
//                 RuleArg::ByRule(arg) => VarMapValue::Expr(CapturedExpr {
//                     expr: arg,
//                     blocks: ByAddress(blocks),
//                     rule_args,
//                     vars,
//                 }),
//             };
//             result_args.push(arg);
//         }
// 
//         match rule {
//             VarMapValue::Expr(captured) => {
//                 if let RuleExpr::RunVar(rule, old_args) = captured.expr {
//                     let Some(rule) = vars.get(rule) else {
//                         panic!("Tried to run variable `{rule}` as a rule, but it was not defined.");
//                     };
// 
//                     presult
//                         .merge_seq_parser(
//                             &parser_rule(rules, rule, &result_args),
//                             state,
//                             context,
//                         )
//                         .map(|(_, v)| PR::with_cow_rtrn(Cow::Borrowed(v)))
//                 } else {
//                     parser_expr(
//                         rules,
//                         captured.blocks.as_ref(),
//                         captured.expr,
//                         captured.rule_args,
//                         captured.vars,
//                     )
//                         .parse(pos, state, context)
//                 }
//             }
//             VarMapValue::Value(value) => {
//                 if let ActionResult::RuleId(rule) = value.as_ref() {
//                     presult
//                         .merge_seq_parser(
//                             &parser_rule(rules, *rule, &result_args),
//                             state,
//                             context,
//                         )
//                         .map(|(_, v)| PR::with_cow_rtrn(Cow::Borrowed(v)))
//                 } else {
//                     let end_pos = presult.end_pos();
//                     presult
//                         .merge_seq(PResult::new_ok(
//                             PR::with_cow_rtrn(value.clone()),
//                             end_pos,
//                             end_pos,
//                         ))
//                         .map(|(_, v)| v)
//                 }
//             }
//         }
//     }
// }

pub fn parser_expr<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
    rules: &'arn GrammarState<'arn, 'grm>,
    block_ctx: BlockCtx<'arn, 'grm>,
    expr: &'arn RuleExpr<'grm, RuleAction<'arn, 'grm>>,
    vars: VarMap<'arn, 'grm>,
) -> impl Parser<'arn, 'grm, PR<'arn, 'grm>, E> + 'a {
    move |pos: Pos,
          state: &mut PState<'arn, 'grm, E>,
          context: ParserContext|
          -> PResult<PR<'arn, 'grm>, E> {
        match expr {
            RuleExpr::RunVar(rule, args) => {
                // Figure out which rule the variable `rule` refers to
                let Some(rule) = vars.get(rule) else {
                    panic!("Tried to run variable `{rule}` as a rule, but it was not defined.");
                };

                let mut presult = PResult::new_ok((), pos, pos);
                let mut result_args: Vec<VarMapValue> = vec![];
                for arg in args {
                    let arg = match arg {
                        RuleArg::ByValue(arg) => VarMapValue::Expr(CapturedExpr {
                            expr: arg,
                            block_ctx: block_ctx,
                            vars,
                        }),
                        RuleArg::ByRule(arg) => VarMapValue::Expr(CapturedExpr {
                            expr: arg,
                            block_ctx: block_ctx,
                            vars,
                        }),
                    };
                    result_args.push(arg);
                }

                match rule {
                    VarMapValue::Expr(captured) => {
                        assert_eq!(
                            result_args.len(),
                            0,
                            "Applying arguments to captured expressions is currently unsupported"
                        );
                        parser_expr(
                            rules,
                            captured.block_ctx,
                            captured.expr,
                            captured.vars,
                        )
                        .parse(pos, state, context)
                    }
                    VarMapValue::Value(value) => {
                        if let ActionResult::RuleId(rule) = value.as_ref() {
                            presult
                                .merge_seq_parser(
                                    &parser_rule(rules, *rule, &result_args),
                                    state,
                                    context,
                                )
                                .map(|(_, v)| PR::with_cow_rtrn(Cow::Borrowed(v)))
                        } else {
                            let end_pos = presult.end_pos();
                            presult
                                .merge_seq(PResult::new_ok(
                                    PR::with_cow_rtrn(value.clone()),
                                    end_pos,
                                    end_pos,
                                ))
                                .map(|(_, v)| v)
                        }
                    }
                }
            }
            RuleExpr::CharClass(cc) => {
                let p = single(|c| cc.contains(*c));
                let p = map_parser(p, &|(span, _)| Cow::Owned(ActionResult::Value(span)));
                let p = recovery_point(p);
                let p = parser_with_layout(rules, vars, &p);
                p.parse(pos, state, context).map(PR::with_cow_rtrn)
            }
            RuleExpr::Literal(literal) => {
                //First construct the literal parser
                let p =
                    move |pos: Pos, state: &mut PState<'arn, 'grm, E>, context: ParserContext| {
                        let mut res = PResult::new_empty((), pos);
                        for char in literal.chars() {
                            res = res
                                .merge_seq_parser(&single(|c| *c == char), state, context)
                                .map(|_| ());
                        }
                        let mut res =
                            res.map_with_span(|_, span| Cow::Owned(ActionResult::Value(span)));
                        res.add_label_implicit(ErrorLabel::Literal(
                            pos.span_to(res.end_pos().next(state.input).0),
                            literal.clone(),
                        ));
                        res
                    };
                let p = recovery_point(p);
                let p = parser_with_layout(rules, vars, &p);
                p.parse(pos, state, context).map(PR::with_cow_rtrn)
            }
            RuleExpr::Repeat {
                expr,
                min,
                max,
                delim,
            } => {
                let res: PResult<Vec<PR>, _> = repeat_delim(
                    parser_expr(rules, block_ctx, expr, vars),
                    parser_expr(rules, block_ctx, delim, vars),
                    *min as usize,
                    max.map(|max| max as usize),
                )
                .parse(pos, state, context);
                res.map_with_span(|list, span| {
                    PR::with_rtrn(ActionResult::Construct(
                        span,
                        "List",
                        list.into_iter().map(|pr| pr.rtrn).collect(),
                    ))
                })
            }
            RuleExpr::Sequence(subs) => {
                let mut res = PResult::new_empty(VarMap::default(), pos);
                //TODO can we do better than tracking res_vars by cloning?
                let mut res_vars: VarMap = vars;
                for sub in subs {
                    res = res
                        .merge_seq_parser(
                            &parser_expr(rules, block_ctx, sub, res_vars),
                            state,
                            context,
                        )
                        .map(|(mut l, r)| {
                            l.extend(r.free.iter_cloned(), &mut state.alloc.alo_varmap)
                        });
                    match &res.ok_ref() {
                        None => break,
                        Some(o) => {
                            res_vars = res_vars.extend(
                                o.iter_cloned(),
                                state.alloc.alo_varmap,
                            );
                        }
                    }
                }
                res.map(|map| PR {
                    free: map,
                    rtrn: Cow::Owned(ActionResult::void()),
                })
            }
            RuleExpr::Choice(subs) => {
                let mut res: PResult<PR, E> = PResult::PErr(E::new(pos.span_to(pos)), pos);
                for sub in subs {
                    res = res.merge_choice_parser(
                        &parser_expr(rules, block_ctx, sub, vars),
                        pos,
                        state,
                        context,
                    );
                    if res.is_ok() {
                        break;
                    }
                }
                res
            }
            RuleExpr::NameBind(name, sub) => {
                let res =
                    parser_expr(rules, block_ctx, sub, vars).parse(pos, state, context);
                res.map(|mut res| {
                    res.free = res.free.insert(name, VarMapValue::Value(res.rtrn.clone()), state.alloc.alo_varmap);
                    res
                })
            }
            RuleExpr::Action(sub, action) => {
                let res =
                    parser_expr(rules, block_ctx, sub, vars).parse(pos, state, context);
                res.map_with_span(|res, span| {
                    let rtrn = apply_action(
                        action,
                        &|k| {
                            match res.free
                                .get(k)
                                .or_else(|| vars.get(k)) {
                                None => None,
                                Some(VarMapValue::Value(ar)) => Some(ar.clone()),
                                Some(VarMapValue::Expr(expr)) => {
                                    // parser_expr(rules, expr.blocks, expr.expr, expr.vars);

                                    todo!()
                                },
                            }
                        },
                        span,
                    );

                    PR {
                        free: res.free,
                        rtrn,
                    }
                })
            }
            RuleExpr::SliceInput(sub) => {
                let res =
                    parser_expr(rules, block_ctx, sub, vars).parse(pos, state, context);
                res.map_with_span(|_, span| PR::with_rtrn(ActionResult::Value(span)))
            }
            RuleExpr::This => parser_body_cache_recurse(rules, block_ctx)
                .parse(pos, state, context)
                .map(|v| PR::with_cow_rtrn(Cow::Borrowed(v))),
            RuleExpr::Next => parser_body_cache_recurse(rules, (ByAddress(&block_ctx.0[1..]), block_ctx.1))
                .parse(pos, state, context)
                .map(|v| PR::with_cow_rtrn(Cow::Borrowed(v))),
            RuleExpr::PosLookahead(sub) => {
                positive_lookahead(&parser_expr(rules, block_ctx, sub, vars))
                    .parse(pos, state, context)
            }
            RuleExpr::NegLookahead(sub) => {
                negative_lookahead(&parser_expr(rules, block_ctx, sub, vars))
                    .parse(pos, state, context)
                    .map(|_| PR::with_rtrn(ActionResult::void()))
            }
            RuleExpr::AtAdapt(ga, adapt_rule) => {
                // First, get the grammar actionresult
                //TODO match this logic with RuleExpr::Action
                let gr = apply_action(
                    ga,
                    &|k| vars.get(k).and_then(|v| v.as_value()).cloned(),
                    Span::invalid(),
                );
                let gr: &'arn ActionResult = state.alloc.uncow(gr);

                // Parse it into a grammar
                //TODO performance: We should have a cache for grammar files
                //TODO and grammar state + new grammar -> grammar state
                let g = match parse_grammarfile(gr, state.input, |ar, _| {
                    Some(RuleAction::ActionResult(ar))
                }) {
                    Some(g) => g,
                    None => {
                        let mut e = E::new(pos.span_to(pos));
                        e.add_label_implicit(ErrorLabel::Explicit(
                            pos.span_to(pos),
                            EscapedString::from_escaped(
                                "language grammar to be correct, but adaptation AST was malformed.",
                            ),
                        ));
                        return PResult::new_err(e, pos);
                    }
                };
                let g: &'arn GrammarFile<'grm, RuleAction<'arn, 'grm>> =
                    state.alloc.alo_grammarfile.alloc(g);

                // Create new grammarstate
                let (rules, rule_vars) = match rules.adapt_with(
                    g,
                    vars,
                    Some(pos),
                    state.alloc.alo_varmap,
                ) {
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
                let rules: &'arn GrammarState = state.alloc.alo_grammarstate.alloc(rules);

                let rule = rule_vars
                    .get(adapt_rule)
                    .or_else(|| vars.get(adapt_rule))
                    .unwrap()
                    .as_rule_id()
                    .expect("Adaptation rule exists");

                // Parse body
                let mut res = parser_rule(rules, rule, &[])
                    .parse(pos, state, context)
                    .map(|v| PR::with_cow_rtrn(Cow::Borrowed(v)));
                res.add_label_implicit(ErrorLabel::Debug(pos.span_to(pos), "adaptation"));
                res
            }
            RuleExpr::Guid => {
                let guid = state.guid_counter;
                state.guid_counter += 1;
                PResult::new_ok(PR::with_rtrn(ActionResult::Guid(guid)), pos, pos)
            }
            RuleExpr::Env => {
                PResult::new_ok(PR::with_rtrn(ActionResult::Env(vars)), pos, pos)
            }
        }
    }
}
