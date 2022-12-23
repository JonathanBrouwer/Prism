use crate::core::parser::{map_parser, Parser};
use crate::core::presult::PResult;
use crate::core::primitives::{negative_lookahead, positive_lookahead, repeat_delim, single};
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::action_result::ActionResult;
use crate::grammar::grammar::{EscapedString, GrammarFile, RuleExpr};
use crate::grammar::parser_layout::parser_with_layout;

use crate::core::adaptive::GrammarState;
use crate::core::context::{Ignore, PCache, ParserContext, PR};
use crate::core::recovery::recovery_point;
use crate::core::stream::StringStream;
use crate::grammar::apply_action::apply_action;
use crate::grammar::from_action_result::parse_grammarfile;
use crate::grammar::parser_rule::parser_rule;
use crate::grammar::parser_rule_body::parser_body_cache_recurse;
use crate::META_GRAMMAR_STATE;
use std::collections::HashMap;
use std::sync::Arc;

pub fn parser_expr<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'b, 'grm>,
    expr: &'b RuleExpr<'grm>,
    vars: &'a HashMap<&'grm str, Arc<ActionResult<'grm>>>,
) -> impl Parser<'b, 'grm, PR<'grm>, E> + 'a {
    move |stream: StringStream<'grm>,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>| {
        match expr {
            RuleExpr::Rule(rule) => parser_rule(rules, rule).parse(stream, cache, context),
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
                let p = move |stream: StringStream<'grm>,
                              cache: &mut PCache<'b, 'grm, E>,
                              context: &ParserContext<'b, 'grm>| {
                    let mut res = PResult::new_ok((), stream);
                    for char in literal.chars() {
                        res = res
                            .merge_seq_parser(&single(|c| *c == char), cache, context)
                            .map(|_| ());
                    }
                    let span = stream.span_to(res.get_stream());
                    let mut res =
                        res.map(|_| (HashMap::new(), Arc::new(ActionResult::Value(span))));
                    res.add_label_implicit(ErrorLabel::Literal(
                        stream.span_to(res.get_stream().next().0),
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
            } => repeat_delim(
                parser_expr(rules, expr, &vars),
                parser_expr(rules, delim, &vars),
                *min as usize,
                max.map(|max| max as usize),
            )
            .parse(stream, cache, context)
            .map(|list| {
                (
                    HashMap::new(),
                    Arc::new(ActionResult::List(
                        list.into_iter().map(|pr| pr.1).collect(),
                    )),
                )
            }),
            RuleExpr::Sequence(subs) => {
                let mut res = PResult::new_ok(HashMap::new(), stream);
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
                let mut res: PResult<'grm, PR, E> =
                    PResult::PErr(E::new(stream.span_to(stream)), stream);
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
                res.map(|mut res| {
                    res.1 = apply_action(action, &res.0);
                    res
                })
            }
            RuleExpr::SliceInput(sub) => {
                let res = parser_expr(rules, sub, vars).parse(stream, cache, context);
                let span = stream.span_to(res.get_stream());
                res.map(|_| (HashMap::new(), Arc::new(ActionResult::Value(span))))
            }
            RuleExpr::AtThis => parser_body_cache_recurse(rules, context.prec_climb_this.unwrap())
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
                .map(|(_, v)| (HashMap::new(), v)),
            RuleExpr::AtNext => parser_body_cache_recurse(rules, context.prec_climb_next.unwrap())
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
                .map(|(_, v)| (HashMap::new(), v)),
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
                parser_rule(&META_GRAMMAR_STATE, "toplevel").parse(stream, cache, context)
            }
            RuleExpr::AtAdapt(ga, b) => {
                // First, get the grammar actionresult
                let gr: Arc<ActionResult<'grm>> = apply_action(ga, vars);

                // Parse it into a grammar
                let g = parse_grammarfile(&*gr, stream.src());
                let g: &'b GrammarFile = cache.alloc.grammarfile_arena.alloc(g);

                // Create new grammarstate
                let mut rules: GrammarState = (*rules).clone();
                if let Err(_) = rules.update(&g) {
                    let mut e = E::new(stream.span_to(stream));
                    e.add_label_implicit(ErrorLabel::Explicit(
                        stream.span_to(stream),
                        EscapedString::from_escaped(
                            "Grammar was invalid, created cycle in block order.",
                        ),
                    ));
                    return PResult::new_err(e, stream);
                }
                let rules: &'b GrammarState = cache.alloc.grammarstate_arena.alloc(rules);

                // Parse body
                parser_rule(&rules, &b[..]).parse(stream, cache, context)
            }
        }
    }
}
