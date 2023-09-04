use crate::grammar::grammar::AnnotatedRuleExpr;
use crate::grammar::grammar::{RuleAnnotation, RuleExpr};
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::cache::{parser_cache_recurse, PCache};
use crate::core::parser::Parser;
use crate::core::presult::PResult;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::parser_layout::parser_with_layout;

use crate::core::adaptive::{BlockState, GrammarState};
use by_address::ByAddress;

use crate::core::context::{Ignore, ParserContext, PR};
use crate::core::pos::Pos;
use crate::core::recovery::recovery_point;
use crate::grammar::parser_rule_expr::parser_expr;

pub fn parser_body_cache_recurse<
    'a,
    'b: 'a,
    'grm: 'b,
    E: ParseError<L = ErrorLabel<'grm>> + Clone,
>(
    rules: &'b GrammarState<'b, 'grm>,
    bs: &'b [BlockState<'b, 'grm>],
    vars: &'a HashMap<&'grm str, Arc<PR<'b, 'grm>>>,
) -> impl Parser<'b, 'grm, PR<'b, 'grm>, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E>, context: &ParserContext<'b, 'grm>| {
        parser_cache_recurse(
            &parser_body_sub_blocks(rules, bs, vars),
            (ByAddress(bs), context.clone()),
        )
        .parse(stream, cache, context)
    }
}

fn parser_body_sub_blocks<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'b, 'grm>,
    bs: &'b [BlockState<'b, 'grm>],
    vars: &'a HashMap<&'grm str, Arc<PR<'b, 'grm>>>,
) -> impl Parser<'b, 'grm, PR<'b, 'grm>, E> + 'a {
    move |stream: Pos,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>|
          -> PResult<PR, E> {
        match bs {
            [] => unreachable!(),
            [b] => parser_body_sub_constructors(rules, &b.constructors[..], vars).parse(
                stream,
                cache,
                &ParserContext {
                    prec_climb_this: Ignore(Some(bs)),
                    prec_climb_next: Ignore(None),
                    ..context.clone()
                },
            ),
            [b, brest @ ..] => {
                // Parse current
                let res = parser_body_sub_constructors(rules, &b.constructors[..], vars).parse(
                    stream,
                    cache,
                    &ParserContext {
                        prec_climb_this: Ignore(Some(bs)),
                        prec_climb_next: Ignore(Some(brest)),
                        ..context.clone()
                    },
                );

                // Parse next with recursion check
                res.merge_choice_parser(
                    &parser_body_cache_recurse(rules, brest, vars),
                    stream,
                    cache,
                    context,
                )
            }
        }
    }
}

fn parser_body_sub_constructors<
    'a,
    'b: 'a,
    'grm: 'b,
    E: ParseError<L = ErrorLabel<'grm>> + Clone,
>(
    rules: &'b GrammarState<'b, 'grm>,
    es: &'b [&'b AnnotatedRuleExpr<'grm>],
    vars: &'a HashMap<&'grm str, Arc<PR<'b, 'grm>>>,
) -> impl Parser<'b, 'grm, PR<'b, 'grm>, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E>, context: &ParserContext<'b, 'grm>| match es
    {
        [] => PResult::new_err(E::new(stream.span_to(stream)), stream),
        [AnnotatedRuleExpr(annots, expr)] => {
            parser_body_sub_annotations(rules, annots, expr, vars).parse(stream, cache, context)
        }
        [AnnotatedRuleExpr(annots, expr), rest @ ..] => {
            parser_body_sub_annotations(rules, annots, expr, vars)
                .parse(stream, cache, context)
                .merge_choice_parser(
                    &parser_body_sub_constructors(rules, rest, vars),
                    stream,
                    cache,
                    context,
                )
        }
    }
}

fn parser_body_sub_annotations<
    'a,
    'b: 'a,
    'grm: 'b,
    E: ParseError<L = ErrorLabel<'grm>> + Clone,
>(
    rules: &'b GrammarState<'b, 'grm>,
    annots: &'b [RuleAnnotation<'grm>],
    expr: &'b RuleExpr<'grm>,
    vars: &'a HashMap<&'grm str, Arc<PR<'b, 'grm>>>,
) -> impl Parser<'b, 'grm, PR<'b, 'grm>, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E>, context: &ParserContext<'b, 'grm>| {
        match annots {
            [RuleAnnotation::Error(err_label), rest @ ..] => {
                let mut res = parser_body_sub_annotations(rules, rest, expr, vars)
                    .parse(stream, cache, context);
                res.add_label_explicit(ErrorLabel::Explicit(
                    stream.span_to(res.end_pos().next(cache.input).0),
                    err_label.clone(),
                ));
                res
            }
            [RuleAnnotation::DisableLayout, rest @ ..] => {
                parser_with_layout(rules, &move |stream: Pos,
                                                 cache: &mut PCache<'b, 'grm, E>,
                                                 context: &ParserContext<'b, 'grm>|
                      -> PResult<_, E> {
                    parser_body_sub_annotations(rules, rest, expr, vars).parse(
                        stream,
                        cache,
                        &ParserContext {
                            layout_disabled: true,
                            ..context.clone()
                        },
                    )
                })
                .parse(stream, cache, context)
            }
            [RuleAnnotation::EnableLayout, rest @ ..] => {
                parser_with_layout(rules, &move |stream: Pos,
                                                 cache: &mut PCache<'b, 'grm, E>,
                                                 context: &ParserContext<'b, 'grm>|
                      -> PResult<_, E> {
                    parser_body_sub_annotations(rules, rest, expr, vars).parse(
                        stream,
                        cache,
                        &ParserContext {
                            layout_disabled: false,
                            ..context.clone()
                        },
                    )
                })
                .parse(stream, cache, context)
            }
            [RuleAnnotation::DisableRecovery, rest @ ..] => recovery_point(
                move |stream: Pos,
                      cache: &mut PCache<'b, 'grm, E>,
                      context: &ParserContext<'b, 'grm>| {
                    parser_body_sub_annotations(rules, rest, expr, vars).parse(
                        stream,
                        cache,
                        &ParserContext {
                            recovery_disabled: true,
                            ..context.clone()
                        },
                    )
                },
            )
            .parse(stream, cache, context),
            [RuleAnnotation::EnableRecovery, rest @ ..] => {
                parser_body_sub_annotations(rules, rest, expr, vars).parse(
                    stream,
                    cache,
                    &ParserContext {
                        recovery_disabled: false,
                        ..context.clone()
                    },
                )
            }
            &[] => parser_expr(rules, expr, vars).parse(stream, cache, context),
        }
    }
}
