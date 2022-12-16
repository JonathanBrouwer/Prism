use crate::grammar::{EscapedString, RuleExpr};
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
use crate::parser_core::parser_cache::ParserCache;
use crate::parser_core::stream::StringStream;
use crate::parser_sugar::apply_action::apply_action;
use crate::parser_sugar::parser_rule::parser_rule;
use crate::parser_sugar::parser_rule_body::parser_body_cache_recurse;
use crate::META_GRAMMAR_STATE;
use std::collections::HashMap;
use std::sync::Arc;
use crate::parser_sugar::parser_context::{Ignore, ParserContext, PR, PState};

pub fn parser_expr<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'b, 'grm>,
    expr: &'b RuleExpr<'grm>,
    vars: &'a HashMap<&'grm str, Arc<ActionResult<'grm>>>,
) -> impl Parser<'b, 'grm, PR<'grm>, E, PState<'b, 'grm, E>> + 'a {
    move |stream: StringStream<'grm>,
          cache: &mut PState<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>| {
        match expr {
            RuleExpr::Rule(rule) => parser_rule(rules, rule).parse(stream, cache, context),
            RuleExpr::CharClass(cc) => parser_with_layout(rules, &single(|c| cc.contains(*c)))
                .parse(stream, cache, context)
                .map(|(span, _)| (HashMap::new(), Arc::new(ActionResult::Value(span)))),
            RuleExpr::Literal(literal) => {
                //First construct the literal parser
                let parser_literal =
                    move |stream: StringStream<'grm>,
                          cache: &mut PState<'b, 'grm, E>,
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
                //Next, allow there to be layout before the literal
                let res = parser_with_layout(rules, &parser_literal).parse(stream, cache, context);
                res
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
                    if let ActionResult::Void(v) = *res.1 {
                        panic!("Tried to bind a void value '{v}' with name '{name}'")
                    }
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

                // Create new grammarstate
                let mut rules: GrammarState = (*rules).clone();
                if let Err(_) = rules.update(&g) {
                    let mut e = E::new(stream.span_to(stream));
                    e.add_label_implicit(ErrorLabel::Explicit(
                        stream.span_to(stream),
                        EscapedString::new_borrow(
                            "Grammar was invalid, created cycle in block order.",
                        ),
                    ));
                    return PResult::new_err(e, stream);
                }

                let mut cache = ParserCache::new();

                let p: PResult<'grm, PR, E> =
                    parser_rule(&rules, &b[..]).parse(stream, &mut cache, context);
                p
            }
        }
    }
}
