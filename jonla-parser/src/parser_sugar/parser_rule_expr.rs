use crate::grammar::{RuleExpr};
use crate::parser_core::error::ParseError;
use crate::parser_core::parser::Parser;
use crate::parser_core::presult::PResult;
use crate::parser_core::primitives::{
    negative_lookahead, positive_lookahead, repeat_delim, single,
};
use crate::parser_sugar::action_result::ActionResult;
use crate::parser_sugar::error_printer::ErrorLabel;
use crate::parser_sugar::parser_layout::parser_with_layout;

use crate::from_action_result::parse_grammarfile;
use crate::parser_core::adaptive::GrammarState;
use crate::parser_core::stream::StringStream;
use crate::parser_sugar::apply_action::apply_action;
use crate::parser_sugar::parser_rule::{parser_rule, PState, ParserContext, PR};
use crate::parser_sugar::parser_rule_body::parser_body_cache_recurse;
use crate::META_GRAMMAR_STATE;
use std::collections::HashMap;
use std::sync::Arc;

pub fn parser_expr<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'b, 'grm>,
    expr: &'b RuleExpr<'grm>,
    context: &'a ParserContext<'b, 'grm>,
    vars: &'a HashMap<&'grm str, Arc<ActionResult<'grm>>>,
) -> impl Parser<'grm, PR<'grm>, E, PState<'b, 'grm, E>> + 'a {
    move |stream: StringStream<'grm>, state: &mut PState<'b, 'grm, E>| {
        match expr {
            RuleExpr::Rule(rule) => parser_rule(rules, rule, context).parse(stream, state),
            RuleExpr::CharClass(cc) => {
                parser_with_layout(rules, &single(|c| cc.contains(*c)), context)
                    .parse(stream, state)
                    .map(|(span, _)| (HashMap::new(), Arc::new(ActionResult::Value(span))))
            }
            RuleExpr::Literal(literal) => {
                //First construct the literal parser
                let parser_literal =
                    move |stream: StringStream<'grm>, state: &mut PState<'b, 'grm, E>| {
                        let mut res = PResult::new_ok((), stream);
                        for char in literal.chars() {
                            res = res
                                .merge_seq_parser(&single(|c| *c == char), state)
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
                //Next, allow there to be layout before the literal
                let res = parser_with_layout(rules, &parser_literal, context).parse(stream, state);
                res
            }
            RuleExpr::Repeat {
                expr,
                min,
                max,
                delim,
            } => repeat_delim(
                parser_expr(rules, expr, context, &vars),
                parser_expr(rules, delim, context, &vars),
                *min as usize,
                max.map(|max| max as usize),
            )
            .parse(stream, state)
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
                        .merge_seq_parser(&parser_expr(rules, sub, context, &res_vars), state)
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
                        &parser_expr(rules, sub, context, vars),
                        stream,
                        state,
                    );
                    if res.is_ok() {
                        break;
                    }
                }
                res
            }
            RuleExpr::NameBind(name, sub) => {
                let res = parser_expr(rules, sub, context, vars).parse(stream, state);
                res.map(|mut res| {
                    if let ActionResult::Void(v) = *res.1 {
                        panic!("Tried to bind a void value '{v}' with name '{name}'")
                    }
                    res.0.insert(name, res.1.clone());
                    res
                })
            }
            RuleExpr::Action(sub, action) => {
                let res = parser_expr(rules, sub, context, vars).parse(stream, state);
                res.map(|mut res| {
                    res.1 = apply_action(action, &res.0);
                    res
                })
            }
            RuleExpr::SliceInput(sub) => {
                let res = parser_expr(rules, sub, context, vars).parse(stream, state);
                let span = stream.span_to(res.get_stream());
                res.map(|_| (HashMap::new(), Arc::new(ActionResult::Value(span))))
            }
            RuleExpr::AtThis => parser_body_cache_recurse(
                rules,
                *context.prec_climb_this.unwrap(),
                // Reset this/next as they shouldn't matter from now on
                &ParserContext {
                    prec_climb_this: None,
                    prec_climb_next: None,
                    ..*context
                },
            )
            .parse(stream, state)
            .map(|(_, v)| (HashMap::new(), v)),
            RuleExpr::AtNext => parser_body_cache_recurse(
                rules,
                *context.prec_climb_next.unwrap(),
                // Reset this/next as they shouldn't matter from now on
                &ParserContext {
                    prec_climb_this: None,
                    prec_climb_next: None,
                    ..*context
                },
            )
            .parse(stream, state)
            .map(|(_, v)| (HashMap::new(), v)),
            RuleExpr::PosLookahead(sub) => {
                positive_lookahead(&parser_expr(rules, sub, context, vars))
                    .parse(stream, state)
                    .map(|r| (HashMap::new(), r.1))
            }
            RuleExpr::NegLookahead(sub) => {
                negative_lookahead(&parser_expr(rules, sub, context, vars))
                    .parse(stream, state)
                    .map(|_| {
                        (
                            HashMap::new(),
                            Arc::new(ActionResult::Void("negative lookahead")),
                        )
                    })
            }
            RuleExpr::AtGrammar => parser_rule(
                &META_GRAMMAR_STATE,
                "toplevel",
                &ParserContext {
                    prec_climb_this: None,
                    prec_climb_next: None,
                    ..*context
                },
            )
            .parse(stream, state),
            RuleExpr::AtAdapt(ga, b) => {
                // let gr: Arc<ActionResult<'grm>> = apply_action(ga, vars);
                // let g = parse_grammarfile(&*gr, stream.src());
                //
                //TODO temp fix, don't leak things pls

                // let g: &'static GrammarFile = Box::leak(Box::new(g));
                // let mut rules: GrammarState = (*rules).clone();
                // if let Err(_) = rules.update(g) {
                //     let mut e = E::new(stream.span_to(stream));
                //     e.add_label_implicit(ErrorLabel::Explicit(
                //         stream.span_to(stream),
                //         "Grammar was invalid, created cycle in block order.",
                //     ));
                //     return PResult::new_err(e, stream);
                // }
                // let rules: &'static GrammarState<'_, 'grm> = Box::leak(Box::new(rules));
                //
                // let p: PResult<'grm, PR, E> =
                //     parser_rule(&rules, &b[..], &context).parse(stream, state);
                // p
                todo!()
            }
        }
    }
}
