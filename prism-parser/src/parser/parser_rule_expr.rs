use crate::core::adaptive::{BlockState, GrammarState};
use crate::core::cache::PCache;
use crate::core::context::{ParserContext, PR};
use crate::core::parser::{map_parser, Parser};
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::primitives::{negative_lookahead, positive_lookahead, repeat_delim, single};
use crate::core::recovery::recovery_point;
use crate::core::span::Span;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::from_action_result::parse_grammarfile;
use crate::grammar::grammar_ar::{GrammarFile, RuleExpr};
use crate::parser::parser_layout::parser_with_layout;
use crate::parser::parser_rule::parser_rule;
use crate::parser::parser_rule_body::parser_body_cache_recurse;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::apply_action::apply_action;
use crate::rule_action::RuleAction;
use std::collections::HashMap;

pub fn parser_expr<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
    rules: &'b GrammarState<'b, 'grm>,
    blocks: &'b [BlockState<'b, 'grm>],
    expr: &'b RuleExpr<'b, 'grm>,
    vars: &'a HashMap<&'grm str, ActionResult<'b, 'grm>>,
) -> impl Parser<'b, 'grm, PR<'b, 'grm>, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E>, context: &ParserContext| {
        match expr {
            RuleExpr::Rule(rule, args) => {
                // Does `rule` refer to a variable containing a rule or to a rule directly?
                let rule = if let Some(ar) = vars.get(rule) {
                    if let ActionResult::RuleRef(rule) = ar {
                        *rule
                    } else {
                        panic!("Tried to run variable `{rule}` as a rule, but it does not refer to a rule. {ar:?}");
                    }
                } else {
                    panic!("Tried to run variable `{rule}` as a rule, but it was not defined.");
                };

                let args = args
                    .iter()
                    .map(|arg| apply_action(arg, &|v| vars.get(v).cloned(), Span::invalid()))
                    .collect::<Vec<_>>();

                let res = parser_rule(rules, rule, &args).parse(stream, cache, context);
                res
            }
            RuleExpr::CharClass(cc) => {
                let p = single(|c| cc.contains(*c));
                let p = map_parser(p, &|(span, _)| PR::with_rtrn(ActionResult::Value(span)));
                let p = recovery_point(p);
                let p = parser_with_layout(rules, vars, &p);
                p.parse(stream, cache, context)
            }
            RuleExpr::Literal(literal) => {
                //First construct the literal parser
                let p =
                    move |stream: Pos, cache: &mut PCache<'b, 'grm, E>, context: &ParserContext| {
                        let mut res = PResult::new_empty((), stream);
                        for char in literal.chars() {
                            res = res
                                .merge_seq_parser(&single(|c| *c == char), cache, context)
                                .map(|_| ());
                        }
                        let mut res =
                            res.map_with_span(|_, span| PR::with_rtrn(ActionResult::Value(span)));
                        res.add_label_implicit(ErrorLabel::Literal(
                            stream.span_to(res.end_pos().next(cache.input).0),
                            literal.clone(),
                        ));
                        res
                    };
                let p = recovery_point(p);
                let p = parser_with_layout(rules, vars, &p);
                p.parse(stream, cache, context)
            }
            RuleExpr::Repeat {
                expr,
                min,
                max,
                delim,
            } => {
                let res: PResult<Vec<PR>, _> = repeat_delim(
                    parser_expr(rules, blocks, expr, vars),
                    parser_expr(rules, blocks, delim, vars),
                    *min as usize,
                    max.map(|max| max as usize),
                )
                .parse(stream, cache, context);
                res.map_with_span(|list, span| {
                    PR::with_rtrn(ActionResult::Construct(
                        span,
                        "List",
                        list.into_iter().map(|pr| pr.rtrn).collect(),
                    ))
                })
            }
            RuleExpr::Sequence(subs) => {
                let mut res = PResult::new_empty(HashMap::new(), stream);
                //TODO can we do better than tracking res_vars by cloning?
                let mut res_vars = vars.clone();
                for sub in subs {
                    res = res
                        .merge_seq_parser(
                            &parser_expr(rules, blocks, sub, &res_vars),
                            cache,
                            context,
                        )
                        .map(|(mut l, r)| {
                            l.extend(r.free);
                            l
                        });
                    match &res.ok() {
                        None => break,
                        Some(o) => {
                            res_vars.extend(o.iter().map(|(k, v)| (*k, v.clone())));
                        }
                    }
                }
                res.map(|map| PR {
                    free: map,
                    rtrn: ActionResult::void(),
                })
            }
            RuleExpr::Choice(subs) => {
                let mut res: PResult<PR, E> = PResult::PErr(E::new(stream.span_to(stream)), stream);
                for sub in subs {
                    res = res.merge_choice_parser(
                        &parser_expr(rules, blocks, sub, vars),
                        stream,
                        cache,
                        context,
                    );
                    if res.is_ok() {
                        break;
                    }
                }
                res
            }
            RuleExpr::NameBind(name, sub) => {
                let res = parser_expr(rules, blocks, sub, vars).parse(stream, cache, context);
                res.map(|mut res| {
                    res.free.insert(name, res.rtrn.clone());
                    res
                })
            }
            RuleExpr::Action(sub, action) => {
                let res = parser_expr(rules, blocks, sub, vars).parse(stream, cache, context);
                res.map(|res| {
                    let rtrn: ActionResult<'b, 'grm> = apply_action(
                        action,
                        &|k| {
                            res.free
                                .get(k)
                                .cloned()
                                .or_else(|| vars.get(k).map(|v| (*v).clone()))
                        },
                        Span::invalid(),
                    );

                    PR {
                        free: res.free,
                        rtrn,
                    }
                })
            }
            RuleExpr::SliceInput(sub) => {
                let res = parser_expr(rules, blocks, sub, vars).parse(stream, cache, context);
                res.map_with_span(|_, span| PR::with_rtrn(ActionResult::Value(span)))
            }
            RuleExpr::AtThis => parser_body_cache_recurse(rules, blocks, vars)
                .parse(stream, cache, context)
                .map(|pr| pr.fresh()),
            RuleExpr::AtNext => parser_body_cache_recurse(rules, &blocks[1..], vars)
                .parse(stream, cache, context)
                .map(|pr| pr.fresh()),
            RuleExpr::PosLookahead(sub) => {
                positive_lookahead(&parser_expr(rules, blocks, sub, vars))
                    .parse(stream, cache, context)
            }
            RuleExpr::NegLookahead(sub) => {
                negative_lookahead(&parser_expr(rules, blocks, sub, vars))
                    .parse(stream, cache, context)
                    .map(|_| PR::with_rtrn(ActionResult::void()))
            }
            RuleExpr::AtAdapt(ga, b) => {
                // First, get the grammar actionresult
                let gr: ActionResult<'b, 'grm> =
                    apply_action(ga, &|k| vars.get(k).cloned(), Span::invalid());
                let gr: &'b ActionResult = cache.alloc.alo_ar.alloc(gr);

                // Parse it into a grammar
                let g = match parse_grammarfile(gr, cache.input, convert_action_result) {
                    Some(g) => g,
                    None => {
                        let mut e = E::new(stream.span_to(stream));
                        e.add_label_implicit(ErrorLabel::Explicit(
                            stream.span_to(stream),
                            EscapedString::from_escaped(
                                "language grammar to be correct, but adaptation AST was malformed.",
                            ),
                        ));
                        return PResult::new_err(e, stream);
                    }
                };
                let g: &'b GrammarFile<'b, 'grm> = cache.alloc.alo_grammarfile.alloc(g);

                // Create new grammarstate
                let (rules, mut iter) = match rules.with(g, vars, Some(stream)) {
                    Ok(rules) => rules,
                    Err(_) => {
                        let mut e = E::new(stream.span_to(stream));
                        e.add_label_implicit(ErrorLabel::Explicit(
                            stream.span_to(stream),
                            EscapedString::from_escaped(
                                "language grammar to be correct, but adaptation created cycle in block order.",
                            ),
                        ));
                        return PResult::new_err(e, stream);
                    }
                };
                let rules: &'b GrammarState = cache.alloc.alo_grammarstate.alloc(rules);

                let rule = iter
                    .find(|(k, _)| k == b)
                    .map(|(_, v)| v)
                    .unwrap_or_else(|| match vars[b] {
                        ActionResult::RuleRef(r) => r,
                        _ => panic!("Adaptation rule not found."),
                    });

                // Parse body
                parser_rule(rules, rule, &[]).parse(stream, cache, context)
            }
        }
    }
}

fn convert_action_result<'grm, 'b>(
    ar: &'b ActionResult<'b, 'grm>,
    _src: &'grm str,
) -> Option<RuleAction<'b, 'grm>> {
    Some(RuleAction::ActionResult(ar.clone()))
}
