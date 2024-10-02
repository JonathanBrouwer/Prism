use crate::core::adaptive::GrammarState;
use crate::core::context::{ParserContext, PR};
use crate::core::parser::{map_parser, Parser};
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::error::error_printer::ErrorLabel;
use crate::error::error_printer::ErrorLabel::Debug;
use crate::error::ParseError;
use crate::grammar::action_result::ActionResult;
use crate::grammar::apply_action::apply_action;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::from_action_result::parse_grammarfile;
use crate::grammar::rule_action::RuleAction;
use crate::grammar::{GrammarFile, RuleExpr};
use crate::parser::parser_rule::parser_rule;
use crate::parser::parser_rule_body::parser_body_cache_recurse;
use crate::parser::var_map::{BlockCtx, CapturedExpr, VarMap, VarMapValue};

pub fn parser_expr<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>>(
    rules: &'arn GrammarState<'arn, 'grm>,
    block_ctx: BlockCtx<'arn, 'grm>,
    expr: &'arn RuleExpr<'arn, 'grm>,
    vars: VarMap<'arn, 'grm>,
) -> impl Parser<'arn, 'grm, PR<'arn, 'grm>, E> + 'a {
    move |pos: Pos,
          state: &mut ParserState<'arn, 'grm, E>,
          context: ParserContext|
          -> PResult<PR<'arn, 'grm>, E> {
        match expr {
            RuleExpr::RunVar(mut rule_str, args) => {
                let mut result_args = vec![];
                let mut args = args;
                let mut block_ctx = block_ctx;
                let mut vars = vars;

                loop {
                    // Figure out which rule the variable `rule` refers to
                    let Some(rule) = vars.get(rule_str) else {
                        panic!(
                            "Tried to run variable `{rule_str}` as a rule, but it was not defined."
                        );
                    };

                    result_args.splice(
                        0..0,
                        args.iter().map(|arg| {
                            VarMapValue::Expr(CapturedExpr {
                                expr: arg,
                                block_ctx,
                                vars,
                            })
                        }),
                    );

                    return match rule {
                        VarMapValue::Expr(captured) => {
                            // If the `Expr` we call is a rule, we might be using it as a higher-order rule
                            // We process this rule in a loop, using the context of the captured expression
                            if let RuleExpr::RunVar(sub_rule, sub_args) = captured.expr {
                                rule_str = sub_rule;
                                args = sub_args;
                                block_ctx = captured.block_ctx;
                                vars = captured.vars;
                                continue;
                            } else {
                                assert_eq!(
                                    result_args.len(),
                                    0,
                                    "Tried to apply an argument to a non-rule expr"
                                );
                                parser_expr(rules, captured.block_ctx, captured.expr, captured.vars)
                                    .parse(pos, state, context)
                            }
                        }
                        VarMapValue::Value(value) => {
                            if let ActionResult::RuleId(rule) = value {
                                parser_rule(rules, *rule, &result_args)
                                    .parse(pos, state, context)
                                    .map(PR::with_rtrn)
                            } else {
                                //TODO remove this code and replace with $value expressions
                                PResult::new_empty(PR::with_rtrn(value), pos)
                            }
                        }
                    };
                }
            }
            RuleExpr::CharClass(cc) => state
                .parse_with_layout(
                    rules,
                    vars,
                    |state, pos| state.parse_char(|c| cc.contains(*c), pos),
                    pos,
                    context,
                )
                .map(|(span, _)| PR::with_rtrn(&*state.alloc.alloc(ActionResult::Value(span)))),
            RuleExpr::Literal(literal) => state.parse_with_layout(
                rules,
                vars,
                |state, start_pos| {
                    let mut res = PResult::new_empty((), start_pos);
                    for char in literal.chars() {
                        let new_res = state.parse_char(|c| *c == char, res.end_pos());
                        res = res.merge_seq(new_res).map(|_| ());
                    }
                    let span = start_pos.span_to(res.end_pos());
                    let mut res =
                        res.map(|_| PR::with_rtrn(&*state.alloc.alloc(ActionResult::Value(span))));
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
                let item = parser_expr(rules, block_ctx, expr, vars);
                let delimiter = parser_expr(rules, block_ctx, delim, vars);

                let mut res: PResult<Vec<_>, E> = PResult::new_empty(vec![], pos);

                for i in 0..max.unwrap_or(u64::MAX) {
                    let pos = res.end_pos();
                    let part = if i == 0 {
                        item.parse(pos, state, context)
                    } else {
                        delimiter
                            .parse(pos, state, context)
                            .merge_seq_parser(&item, state, context)
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
                        e.add_label_explicit(Debug(span, "INFLOOP"));
                        return PResult::new_err(e, pos);
                    }
                }

                res.map_with_span(|rtrn, span| {
                    PR::with_rtrn(rtrn.iter().rfold(
                        state.alloc.alloc(ActionResult::Construct(span, "Nil", &[])),
                        |rest, next| {
                            state.alloc.alloc(ActionResult::Construct(
                                span,
                                "Cons",
                                state.alloc.alloc_extend([*next.rtrn, *rest]),
                            ))
                        },
                    ))
                })
            }
            RuleExpr::Sequence(subs) => {
                let mut res = PResult::new_empty(VarMap::default(), pos);
                //TODO can we do better than tracking res_vars by cloning?
                let mut res_vars: VarMap = vars;
                for sub in *subs {
                    res = res
                        .merge_seq_parser(
                            &parser_expr(rules, block_ctx, sub, res_vars),
                            state,
                            context,
                        )
                        .map(|(l, r)| l.extend(r.free.iter_cloned(), state.alloc));
                    match &res.ok_ref() {
                        None => break,
                        Some(o) => {
                            res_vars = res_vars.extend(o.iter_cloned(), state.alloc);
                        }
                    }
                }
                res.map(|map| PR {
                    free: map,
                    rtrn: ActionResult::VOID,
                })
            }
            RuleExpr::Choice(subs) => {
                let mut res: PResult<PR, E> = PResult::PErr(E::new(pos.span_to(pos)), pos);
                for sub in *subs {
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
                let res = parser_expr(rules, block_ctx, sub, vars).parse(pos, state, context);
                res.map(|res| PR {
                    free: res
                        .free
                        .insert(name, VarMapValue::Value(res.rtrn), state.alloc),
                    rtrn: ActionResult::VOID,
                })
            }
            RuleExpr::Action(sub, action) => {
                let res = parser_expr(rules, block_ctx, sub, vars).parse(pos, state, context);
                res.map_with_span(|res, span| {
                    let rtrn = apply_action(
                        action,
                        span,
                        res.free.extend(vars.iter_cloned(), state.alloc),
                        &state.alloc,
                    );

                    PR {
                        free: res.free,
                        rtrn: state.alloc.alloc(rtrn),
                    }
                })
            }
            RuleExpr::SliceInput(sub) => {
                let res = parser_expr(rules, block_ctx, sub, vars).parse(pos, state, context);
                res.map_with_span(|_, span| {
                    PR::with_rtrn(state.alloc.alloc(ActionResult::Value(span)))
                })
            }
            RuleExpr::This => parser_body_cache_recurse(rules, block_ctx)
                .parse(pos, state, context)
                .map(PR::with_rtrn),
            RuleExpr::Next => parser_body_cache_recurse(rules, (&block_ctx.0[1..], block_ctx.1))
                .parse(pos, state, context)
                .map(PR::with_rtrn),
            RuleExpr::PosLookahead(sub) => parser_expr(rules, block_ctx, sub, vars)
                .parse(pos, state, context)
                .positive_lookahead(pos),
            RuleExpr::NegLookahead(sub) => parser_expr(rules, block_ctx, sub, vars)
                .parse(pos, state, context)
                .negative_lookahead(pos)
                .map(|()| PR::with_rtrn(ActionResult::VOID)),
            RuleExpr::AtAdapt(ga, adapt_rule) => {
                // First, get the grammar actionresult
                let gr = if let Some(ar) = vars.get(ga) {
                    if let VarMapValue::Value(v) = ar {
                        v
                    } else {
                        panic!("")
                    }
                } else {
                    panic!("Name '{ga}' not in context")
                };

                // Parse it into a grammar
                //TODO performance: We should have a cache for grammar files
                //TODO and grammar state + new grammar -> grammar state
                let g = match parse_grammarfile(gr, state.input, state.alloc, |ar, _| {
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
                let g: &'arn GrammarFile<'arn, 'grm> = state.alloc.alloc(g);

                // Create new grammarstate
                let (rules, _) = match rules.adapt_with(g, vars, Some(pos), state.alloc) {
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
                let rules: &'arn GrammarState = state.alloc.alloc(rules);

                let rule = vars
                    .get(adapt_rule)
                    .or_else(|| vars.get(adapt_rule))
                    .unwrap()
                    .as_rule_id()
                    .expect("Adaptation rule exists");

                // Parse body
                let mut res = parser_rule(rules, rule, &[])
                    .parse(pos, state, context)
                    .map(PR::with_rtrn);
                res.add_label_implicit(ErrorLabel::Debug(pos.span_to(pos), "adaptation"));
                res
            }
            RuleExpr::Guid => {
                let guid = state.guid_counter;
                state.guid_counter += 1;
                PResult::new_empty(
                    PR::with_rtrn(state.alloc.alloc(ActionResult::Guid(guid))),
                    pos,
                )
            }
        }
    }
}
