use crate::grammar::RuleAnnotation;
use crate::grammar::RuleBodyExpr;

use crate::parser::actual::error_printer::ErrorLabel;
use crate::parser::actual::parser_layout::parser_with_layout;
use crate::parser::core::error::ParseError;
use crate::parser::core::parser::Parser;
use crate::parser::core::parser_state::{parser_cache_recurse, ParserState};
use crate::parser::core::presult::PResult;

use crate::parser::core::stream::Stream;
use by_address::ByAddress;

use std::collections::HashMap;

use crate::parser::actual::parser_rule::{ParserContext, PR};
use crate::parser::actual::parser_rule_expr::parser_expr;

pub fn parser_body_cache_recurse<
    'a,
    'b: 'a,
    'grm: 'b,
    S: Stream<I = char>,
    E: ParseError<L = ErrorLabel<'grm>> + Clone,
>(
    rules: &'b HashMap<&'grm str, RuleBodyExpr<'grm>>,
    body: &'b RuleBodyExpr<'grm>,
    context: &'a ParserContext<'b, 'grm>,
) -> impl Parser<char, PR<'grm>, S, E, ParserState<'b, 'grm, PResult<PR<'grm>, E, S>>> + 'a {
    move |stream: S,
          state: &mut ParserState<'b, 'grm, PResult<PR<'grm>, E, S>>|
          -> PResult<PR<'grm>, E, S> {
        parser_cache_recurse(
            &parser_body_sub(rules, body, context),
            (ByAddress(body), context.clone()),
        )
        .parse(stream, state)
    }
}

fn parser_body_sub<
    'a,
    'b: 'a,
    'grm: 'b,
    S: Stream<I = char>,
    E: ParseError<L = ErrorLabel<'grm>> + Clone,
>(
    rules: &'b HashMap<&'grm str, RuleBodyExpr<'grm>>,
    body: &'b RuleBodyExpr<'grm>,
    context: &'a ParserContext<'b, 'grm>,
) -> impl Parser<char, PR<'grm>, S, E, ParserState<'b, 'grm, PResult<PR<'grm>, E, S>>> + 'a {
    move |stream: S,
          state: &mut ParserState<'b, 'grm, PResult<PR<'grm>, E, S>>|
          -> PResult<PR<'grm>, E, S> {
        match body {
            RuleBodyExpr::Body(expr) => parser_expr(rules, expr, context).parse(stream, state),
            RuleBodyExpr::Constructors(c1, c2) => parser_body_sub(rules, c1, context)
                .parse(stream, state)
                .merge_choice_parser(&parser_body_sub(rules, c2, context), stream, state),
            RuleBodyExpr::PrecedenceClimbBlock(e_this, e_next) => {
                //Parse current with recursion check
                let res = parser_body_cache_recurse(
                    rules,
                    e_this,
                    &ParserContext {
                        prec_climb_this: Some(ByAddress(body)),
                        prec_climb_next: Some(ByAddress(e_next)),
                        ..*context
                    },
                )
                .parse(stream, state);
                //Parse next with recursion check
                res.merge_choice_parser(
                    &parser_body_cache_recurse(
                        rules,
                        e_next,
                        &ParserContext {
                            prec_climb_this: None,
                            prec_climb_next: None,
                            ..*context
                        },
                    ),
                    stream,
                    state,
                )
            }
            RuleBodyExpr::Annotation(RuleAnnotation::Error(err_label), rest) => {
                let mut res = parser_body_sub(rules, rest, context).parse(stream, state);
                res.add_label(ErrorLabel::Explicit(
                    stream.span_to(res.get_stream().next().0),
                    err_label,
                ));
                res
            }
            RuleBodyExpr::Annotation(RuleAnnotation::NoLayout, rest) => parser_with_layout(
                rules,
                &move |stream: S,
                       state: &mut ParserState<'b, 'grm, PResult<PR<'grm>, E, S>>|
                      -> PResult<_, E, S> {
                    parser_body_sub(
                        rules,
                        rest,
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
        }
    }
}
