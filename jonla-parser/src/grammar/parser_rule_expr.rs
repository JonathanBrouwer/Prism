use crate::core::parser::{map_parser, Parser};
use crate::core::presult::PResult;
use crate::core::primitives::{negative_lookahead, positive_lookahead, repeat_delim, single};
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::action_result::ActionResult;
use crate::grammar::grammar::{GrammarFile, RuleExpr};
use crate::grammar::parser_layout::parser_with_layout;

use crate::core::adaptive::{GrammarState};
use crate::core::cache::PCache;
use crate::core::context::{Ignore, ParserContext, PR};
use crate::core::pos::Pos;
use crate::core::recovery::recovery_point;
use crate::core::span::Span;
use crate::grammar::apply_action::apply_action;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::from_action_result::parse_grammarfile;
use crate::grammar::parser_rule::parser_rule;
use crate::grammar::parser_rule_body::parser_body_cache_recurse;
use crate::META_GRAMMAR_STATE;
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::Arc;

pub fn parser_expr<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'b, 'grm>,
    expr: &'b RuleExpr<'grm>,
    vars: &'a HashMap<&'grm str, Arc<ActionResult<'grm>>>,
) -> impl Parser<'b, 'grm, PR<'grm>, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E>, context: &ParserContext<'b, 'grm>| {
        match expr {
            RuleExpr::Rule(rule, args) => {
                let arg_func = |n: &str| vars.get(n).map(|v| v.clone()).or(rules.get(n).map(|r| Arc::new(ActionResult::RuleRef(r.name))));
                let args = args
                    .iter()
                    .map(|arg| apply_action(arg, &arg_func, Span::invalid()))
                    .collect_vec();
                let res = parser_rule(rules, rule, &args).parse(stream, cache, context);
                res
            }
            RuleExpr::CharClass(cc) => {
                let p = single(|c| cc.contains(*c));
                let p = map_parser(p, &|(span, _)| {
                    (HashMap::new(), Arc::new(ActionResult::Value(span)))
                });
                let p = recovery_point(p);
                let p = parser_with_layout(rules, &p);
                p.parse(stream, cache, context)
            }
            RuleExpr::Literal(literal) => {
                //First construct the literal parser
                let p = move |stream: Pos,
                              cache: &mut PCache<'b, 'grm, E>,
                              context: &ParserContext<'b, 'grm>| {
                    let mut res = PResult::new_empty((), stream);
                    for char in literal.chars() {
                        res = res
                            .merge_seq_parser(&single(|c| *c == char), cache, context)
                            .map(|_| ());
                    }
                    let mut res = res.map_with_span(|_, span| {
                        (HashMap::new(), Arc::new(ActionResult::Value(span)))
                    });
                    res.add_label_implicit(ErrorLabel::Literal(
                        stream.span_to(res.end_pos().next(cache.input).0),
                        literal.clone(),
                    ));
                    res
                };
                let p = recovery_point(p);
                let p = parser_with_layout(rules, &p);
                p.parse(stream, cache, context)
            }
            RuleExpr::Repeat {
                expr,
                min,
                max,
                delim,
            } => {
                let res = repeat_delim(
                    parser_expr(rules, expr, &vars),
                    parser_expr(rules, delim, &vars),
                    *min as usize,
                    max.map(|max| max as usize),
                )
                .parse(stream, cache, context);
                res.map_with_span(|list, span| {
                    (
                        HashMap::new(),
                        Arc::new(ActionResult::Construct(
                            span,
                            "List",
                            list.into_iter().map(|pr| pr.1).collect(),
                        )),
                    )
                })
            }
            RuleExpr::Sequence(subs) => {
                let mut res = PResult::new_empty(HashMap::new(), stream);
                let mut res_vars = vars.clone();
                for sub in subs {
                    res = res
                        .merge_seq_parser(&parser_expr(rules, sub, &res_vars), cache, context)
                        .map(|(mut l, r)| {
                            l.extend(r.0);
                            l
                        });
                    if res.is_err() {
                        break;
                    }
                    res_vars.extend(res.ok().unwrap().clone().into_iter());
                }
                res.map(|map| (map, Arc::new(ActionResult::Void("sequence"))))
            }
            RuleExpr::Choice(subs) => {
                let mut res: PResult<PR, E> = PResult::PErr(E::new(stream.span_to(stream)), stream);
                for sub in subs {
                    res = res.merge_choice_parser(
                        &parser_expr(rules, sub, vars),
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
                let res = parser_expr(rules, sub, vars).parse(stream, cache, context);
                res.map(|mut res| {
                    res.0.insert(name, res.1.clone());
                    res
                })
            }
            RuleExpr::Action(sub, action) => {
                let res = parser_expr(rules, sub, vars).parse(stream, cache, context);
                res.map_with_span(|mut res, span| {
                    let arg_function = |n: &str| res.0.get(n).or(vars.get(n)).map(|v| v.clone());
                    res.1 = apply_action(action, &arg_function, span);
                    res
                })
            }
            RuleExpr::SliceInput(sub) => {
                let res = parser_expr(rules, sub, vars).parse(stream, cache, context);
                res.map_with_span(|_, span| (HashMap::new(), Arc::new(ActionResult::Value(span))))
            }
            RuleExpr::AtThis => {
                parser_body_cache_recurse(rules, context.prec_climb_this.unwrap(), vars)
                    .parse(
                        stream,
                        cache,
                        // Reset this/next as they shouldn't matter from now on
                        &ParserContext {
                            prec_climb_this: Ignore(None),
                            prec_climb_next: Ignore(None),
                            ..context.clone()
                        },
                    )
                    .map(|(_, v)| (HashMap::new(), v))
            }
            RuleExpr::AtNext => {
                parser_body_cache_recurse(rules, context.prec_climb_next.unwrap(), vars)
                    .parse(
                        stream,
                        cache,
                        // Reset this/next as they shouldn't matter from now on
                        &ParserContext {
                            prec_climb_this: Ignore(None),
                            prec_climb_next: Ignore(None),
                            ..context.clone()
                        },
                    )
                    .map(|(_, v)| (HashMap::new(), v))
            }
            RuleExpr::PosLookahead(sub) => positive_lookahead(&parser_expr(rules, sub, vars))
                .parse(stream, cache, context)
                .map(|r| (HashMap::new(), r.1)),
            RuleExpr::NegLookahead(sub) => negative_lookahead(&parser_expr(rules, sub, vars))
                .parse(stream, cache, context)
                .map(|_| {
                    (
                        HashMap::new(),
                        Arc::new(ActionResult::Void("negative lookahead")),
                    )
                }),
            RuleExpr::AtGrammar => {
                parser_rule(&META_GRAMMAR_STATE, "toplevel", &vec![]).parse(stream, cache, context)
            }
            RuleExpr::AtAdapt(ga, b) => {
                // First, get the grammar actionresult
                let arg_function = |n: &str| vars.get(n).map(|v| v.clone());
                let gr: Arc<ActionResult<'grm>> = apply_action(ga, &arg_function, Span::invalid());

                // Parse it into a grammar
                let g = match parse_grammarfile(&*gr, cache.input) {
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
                let g: &'b GrammarFile = cache.alloc.grammarfile_arena.alloc(g);

                // Create new grammarstate
                let mut rules: GrammarState = (*rules).clone();
                if let Err(_) = rules.update(&g) {
                    let mut e = E::new(stream.span_to(stream));
                    e.add_label_implicit(ErrorLabel::Explicit(
                        stream.span_to(stream),
                        EscapedString::from_escaped(
                            "language grammar to be correct, but adaptation created cycle in block order.",
                        ),
                    ));
                    return PResult::new_err(e, stream);
                }
                let rules: &'b GrammarState = cache.alloc.grammarstate_arena.alloc(rules);

                // Parse body
                parser_rule(&rules, &b[..], &vec![]).parse(stream, cache, context)
            }
        }
    }
}
