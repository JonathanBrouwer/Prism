use crate::grammar::{AnnotatedRuleExpr};
use crate::grammar::{RuleAnnotation, RuleExpr};

use crate::parser_core::error::ParseError;
use crate::parser_core::parser::Parser;
use crate::parser_core::parser_state::parser_cache_recurse;
use crate::parser_core::presult::PResult;
use crate::parser_sugar::error_printer::ErrorLabel;
use crate::parser_sugar::parser_layout::parser_with_layout;

use by_address::ByAddress;
use crate::parser_core::adaptive::{BlockState, GrammarState};

use crate::parser_core::stream::StringStream;
use crate::parser_sugar::parser_rule::{PState, ParserContext, PR};
use crate::parser_sugar::parser_rule_expr::parser_expr;

pub fn parser_body_cache_recurse<
    'a,
    'b: 'a,
    'grm: 'b,
    E: ParseError<L = ErrorLabel<'grm>> + Clone,
>(
    rules: &'b GrammarState<'grm>,
    bs: &'b [BlockState<'grm>],
    context: &'a ParserContext<'b, 'grm>,
) -> impl Parser<'grm, PR<'grm>, E, PState<'b, 'grm, E>> + 'a {
    move |stream: StringStream<'grm>, state: &mut PState<'b, 'grm, E>| {
        parser_cache_recurse(
            &parser_body_sub_blocks(rules, bs, context),
            (ByAddress(bs), context.clone()),
        )
        .parse(stream, state)
    }
}

fn parser_body_sub_blocks<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'grm>,
    bs: &'b [BlockState<'grm>],
    context: &'a ParserContext<'b, 'grm>,
) -> impl Parser<'grm, PR<'grm>, E, PState<'b, 'grm, E>> + 'a {
    move |stream: StringStream<'grm>, state: &mut PState<'b, 'grm, E>| -> PResult<'grm, PR, E> {
        match bs {
            [] => unreachable!(),
            [b] => parser_body_sub_constructors(rules, &b.constructors[..], context).parse(stream, state),
            [b, brest @ ..] => {
                // Parse current
                let res = parser_body_sub_constructors(
                    rules,
                    &b.constructors[..],
                    &ParserContext {
                        prec_climb_this: Some(ByAddress(bs)),
                        prec_climb_next: Some(ByAddress(brest)),
                        ..*context
                    },
                )
                .parse(stream, state);

                // Parse next with recursion check
                res.merge_choice_parser(
                    &parser_body_cache_recurse(rules, brest, context),
                    stream,
                    state,
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
    rules: &'b GrammarState<'grm>,
    es: &'b [&'grm AnnotatedRuleExpr],
    context: &'a ParserContext<'b, 'grm>,
) -> impl Parser<'grm, PR<'grm>, E, PState<'b, 'grm, E>> + 'a {
    move |stream: StringStream<'grm>, state: &mut PState<'b, 'grm, E>| match es {
        [] => unreachable!(),
        [AnnotatedRuleExpr(annots, expr)] => {
            parser_body_sub_annotations(rules, annots, expr, context).parse(stream, state)
        }
        [AnnotatedRuleExpr(annots, expr), rest @ ..] => {
            parser_body_sub_annotations(rules, annots, expr, context)
                .parse(stream, state)
                .merge_choice_parser(
                    &parser_body_sub_constructors(rules, rest, context),
                    stream,
                    state,
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
    rules: &'b GrammarState<'grm>,
    annots: &'grm [RuleAnnotation],
    expr: &'grm RuleExpr,
    context: &'a ParserContext<'b, 'grm>,
) -> impl Parser<'grm, PR<'grm>, E, PState<'b, 'grm, E>> + 'a {
    move |stream: StringStream<'grm>, state: &mut PState<'b, 'grm, E>| match annots {
        [RuleAnnotation::Error(err_label), rest @ ..] => {
            let mut res =
                parser_body_sub_annotations(rules, rest, expr, context).parse(stream, state);
            res.add_label_explicit(ErrorLabel::Explicit(
                stream.span_to(res.get_stream().next().0),
                err_label,
            ));
            res
        }
        [RuleAnnotation::NoLayout, rest @ ..] => parser_with_layout(
            rules,
            &move |stream: StringStream<'grm>,
                   state: &mut PState<'b, 'grm, E>|
                  -> PResult<'grm, _, E> {
                parser_body_sub_annotations(
                    rules,
                    rest,
                    expr,
                    &ParserContext {
                        layout_disabled: true,
                        ..*context
                    },
                )
                .parse(stream, state)
            },
            context,
        )
        .parse(stream, state),
        &[] => parser_expr(rules, expr, context).parse(stream, state),
    }
}