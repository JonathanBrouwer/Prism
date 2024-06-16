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
use crate::grammar::{GrammarFile, RuleExpr};
use crate::parser::parser_layout::parser_with_layout;
use crate::parser::parser_rule::parser_rule;
use crate::parser::parser_rule_body::parser_body_cache_recurse;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::apply_action::apply_action;
use crate::rule_action::RuleAction;
use std::collections::HashMap;
use by_address::ByAddress;
use crate::parser::var_map::{VarMap, VarMapValue};

pub fn parser_expr<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
    rules: &'arn GrammarState<'arn, 'grm>,
    blocks: &'arn [BlockState<'arn, 'grm>],
    expr: &'arn RuleExpr<'grm, RuleAction<'arn, 'grm>>,
    // TODO merge this with `blocks`
    rule_args: &'a VarMap<'arn, 'grm>,
    vars: &'a VarMap<'arn, 'grm>,
) -> impl Parser<'arn, 'grm, PR<'arn, 'grm>, E> + 'a {
    move |pos: Pos,
          state: &mut PState<'arn, 'grm, E>,
          context: &ParserContext|
          -> PResult<PR<'arn, 'grm>, E> {
        match expr {
            RuleExpr::Rule(rule, args) => {
                // Figure out which rule the variable `rule` refers to
                let Some(rule) = vars.get(rule) else {
                    panic!("Tried to run variable `{rule}` as a rule, but it was not defined.");
                };
                match rule {
                    VarMapValue::Expr { expr, blocks, rule_args, vars } => {
                        assert_eq!(args.len(), 0);
                        parser_expr(rules, blocks, expr, rule_args, vars).parse(pos, state, context)
                    }
                    VarMapValue::Value(rule) => {
                        let args = args
                            .iter()
                            .map(|arg| {
                                VarMapValue::Expr {
                                    expr: arg,
                                    blocks: ByAddress(blocks),
                                    rule_args: rule_args.clone(),
                                    vars: vars.clone(),
                                }
                            })
                            .collect::<Vec<_>>();
                        let res = parser_rule(rules, rule.as_rule().unwrap(), &args).parse(pos, state, context);
                        res.map(|v| PR::with_cow_rtrn(Cow::Borrowed(v)))
                    }
                }

                // let rule = rule.to_parser(rules).parse(pos, state, context);

                // rule.merge_seq_chain_parser(
                //     |rule| {
                //         println!("{rule:?}");
                //         let rule = rule.rtrn.as_rule().expect("Value should be a rule");
                //         map_parser(
                //             parser_rule(rules, rule, &args),
                //             &|v| PR::with_cow_rtrn(Cow::Borrowed(v))
                //         )
                //     }
                //     , state, context
                // )
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
                    move |pos: Pos, state: &mut PState<'arn, 'grm, E>, context: &ParserContext| {
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
                    parser_expr(rules, blocks, expr, rule_args, vars),
                    parser_expr(rules, blocks, delim, rule_args, vars),
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
                let mut res = PResult::new_empty(HashMap::new(), pos);
                //TODO can we do better than tracking res_vars by cloning?
                let mut res_vars: VarMap = vars.clone();
                for sub in subs {
                    res = res
                        .merge_seq_parser(
                            &parser_expr(rules, blocks, sub, rule_args, &res_vars),
                            state,
                            context,
                        )
                        .map(|(mut l, r)| {
                            l.extend(r.free);
                            l
                        });
                    match &res.ok() {
                        None => break,
                        Some(o) => {
                            res_vars.extend(o.iter().map(|(k, v)| (*k, VarMapValue::Value(v.clone()))));
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
                        &parser_expr(rules, blocks, sub, rule_args, vars),
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
                    parser_expr(rules, blocks, sub, rule_args, vars).parse(pos, state, context);
                res.map(|mut res| {
                    res.free.insert(name, res.rtrn.clone());
                    res
                })
            }
            RuleExpr::Action(sub, action) => {
                let res =
                    parser_expr(rules, blocks, sub, rule_args, vars).parse(pos, state, context);
                res.map_with_span(|res, span| {
                    let rtrn = apply_action(
                        action,
                        &|k| {
                            res.free
                                .get(k)
                                .cloned()
                                .or_else(|| vars.get(k).and_then(|v| v.as_value()).cloned())
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
                    parser_expr(rules, blocks, sub, rule_args, vars).parse(pos, state, context);
                res.map_with_span(|_, span| PR::with_rtrn(ActionResult::Value(span)))
            }
            RuleExpr::AtThis => parser_body_cache_recurse(rules, blocks, rule_args)
                .parse(pos, state, context)
                .map(|v| PR::with_cow_rtrn(Cow::Borrowed(v))),
            RuleExpr::AtNext => parser_body_cache_recurse(rules, &blocks[1..], rule_args)
                .parse(pos, state, context)
                .map(|v| PR::with_cow_rtrn(Cow::Borrowed(v))),
            RuleExpr::PosLookahead(sub) => {
                positive_lookahead(&parser_expr(rules, blocks, sub, rule_args, vars))
                    .parse(pos, state, context)
            }
            RuleExpr::NegLookahead(sub) => {
                negative_lookahead(&parser_expr(rules, blocks, sub, rule_args, vars))
                    .parse(pos, state, context)
                    .map(|_| PR::with_rtrn(ActionResult::void()))
            }
            RuleExpr::AtAdapt(ga, b) => {
                // First, get the grammar actionresult
                let gr = apply_action(ga, &|k| vars.get(k).and_then(|v| v.as_value()).cloned(), Span::invalid());
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
                let rule_vars = vars.iter().flat_map(|(k, v)| v.as_rule().map(|v| (k, v)));
                let (rules, mut iter) = match rules.adapt_with(g, rule_vars, Some(pos)) {
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

                let rule = iter
                    .find(|(k, _)| k == b)
                    .map(|(_, v)| v)
                    .unwrap_or_else(|| vars.get(b).unwrap().as_rule().expect("Adaptation rule exists"));

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
        }
    }
}
