use crate::core::cow::Cow;
use std::collections::HashMap;

use crate::core::cache::{parser_cache_recurse, PCache};
use crate::core::parser::Parser;
use crate::core::presult::PResult;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;

use crate::core::adaptive::{BlockState, Constructor, GrammarState, RuleId};
use crate::rule_action::RuleAction;
use by_address::ByAddress;

use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::recovery::recovery_point;
use crate::grammar::{RuleAnnotation, RuleExpr};
use crate::parser::parser_layout::parser_with_layout;
use crate::parser::parser_rule_expr::parser_expr;
use crate::rule_action::action_result::ActionResult;

pub fn parser_body_cache_recurse<
    'a,
    'arn: 'a,
    'grm: 'arn,
    E: ParseError<L = ErrorLabel<'grm>> + 'grm,
>(
    rules: &'arn GrammarState<'arn, 'grm>,
    bs: &'arn [BlockState<'arn, 'grm>],
    rule_args: &'a [(&'grm str, RuleId)],
) -> impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'arn, 'grm, E>, context: &ParserContext| {
        parser_cache_recurse(
            &parser_body_sub_blocks(rules, bs, rule_args),
            ByAddress(bs),
            rules.unique_id(),
            rule_args.to_vec(),
        )
        .parse(stream, cache, context)
    }
}

fn parser_body_sub_blocks<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
    rules: &'arn GrammarState<'arn, 'grm>,
    bs: &'arn [BlockState<'arn, 'grm>],
    rule_args: &'a [(&'grm str, RuleId)],
) -> impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E> + 'a {
    move |stream: Pos,
          cache: &mut PCache<'arn, 'grm, E>,
          context: &ParserContext|
          -> PResult<&'arn ActionResult<'arn, 'grm>, E> {
        match bs {
            [] => unreachable!(),
            [b] => parser_body_sub_constructors(rules, bs, &b.constructors[..], rule_args)
                .parse(stream, cache, context),
            [b, brest @ ..] => {
                // Parse current
                let res = parser_body_sub_constructors(rules, bs, &b.constructors[..], rule_args)
                    .parse(stream, cache, context);

                // Parse next with recursion check
                res.merge_choice_parser(
                    &parser_body_cache_recurse(rules, brest, rule_args),
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
    'arn: 'a,
    'grm: 'arn,
    E: ParseError<L = ErrorLabel<'grm>> + 'grm,
>(
    rules: &'arn GrammarState<'arn, 'grm>,
    blocks: &'arn [BlockState<'arn, 'grm>],
    es: &'arn [Constructor<'arn, 'grm>],
    rule_args: &'a [(&'grm str, RuleId)],
) -> impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'arn, 'grm, E>, context: &ParserContext| match es {
        [] => PResult::new_err(E::new(stream.span_to(stream)), stream),
        [(crate::grammar::AnnotatedRuleExpr(annots, expr), rule_ctx), rest @ ..] => {
            let rule_ctx = rule_ctx
                .iter()
                .map(|(&k, v)| (k, Cow::Owned(ActionResult::RuleRef(*v))));
            let rule_args_iter = rule_args
                .iter()
                .map(|&(k, v)| (k, Cow::Owned(ActionResult::RuleRef(v))));
            let vars: HashMap<&'grm str, Cow<'arn, ActionResult>> =
                rule_args_iter.chain(rule_ctx).collect();

            let res = parser_body_sub_annotations(rules, blocks, annots, expr, rule_args, &vars)
                .parse(stream, cache, context)
                .map(|v| cache.alloc.uncow(v))
                .merge_choice_parser(
                    &parser_body_sub_constructors(rules, blocks, rest, rule_args),
                    stream,
                    cache,
                    context,
                );
            res
        }
    }
}

fn parser_body_sub_annotations<
    'a,
    'arn: 'a,
    'grm: 'arn,
    E: ParseError<L = ErrorLabel<'grm>> + 'grm,
>(
    rules: &'arn GrammarState<'arn, 'grm>,
    blocks: &'arn [BlockState<'arn, 'grm>],
    annots: &'arn [RuleAnnotation<'grm>],
    expr: &'arn RuleExpr<'grm, RuleAction<'arn, 'grm>>,
    rule_args: &'a [(&'grm str, RuleId)],
    vars: &'a HashMap<&'grm str, Cow<'arn, ActionResult<'arn, 'grm>>>,
) -> impl Parser<'arn, 'grm, Cow<'arn, ActionResult<'arn, 'grm>>, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'arn, 'grm, E>, context: &ParserContext| match annots {
        [RuleAnnotation::Error(err_label), rest @ ..] => {
            let mut res = parser_body_sub_annotations(rules, blocks, rest, expr, rule_args, vars)
                .parse(stream, cache, context);
            res.add_label_explicit(ErrorLabel::Explicit(
                stream.span_to(res.end_pos().next(cache.input).0),
                err_label.clone(),
            ));
            res
        }
        [RuleAnnotation::DisableLayout, rest @ ..] => {
            parser_with_layout(rules, vars, &move |stream: Pos,
                                                   cache: &mut PCache<'arn, 'grm, E>,
                                                   context: &ParserContext|
                  -> PResult<_, E> {
                parser_body_sub_annotations(rules, blocks, rest, expr, rule_args, vars).parse(
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
            parser_with_layout(rules, vars, &move |stream: Pos,
                                                   cache: &mut PCache<'arn, 'grm, E>,
                                                   context: &ParserContext|
                  -> PResult<_, E> {
                parser_body_sub_annotations(rules, blocks, rest, expr, rule_args, vars).parse(
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
            move |stream: Pos, cache: &mut PCache<'arn, 'grm, E>, context: &ParserContext| {
                parser_body_sub_annotations(rules, blocks, rest, expr, rule_args, vars).parse(
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
            parser_body_sub_annotations(rules, blocks, rest, expr, rule_args, vars).parse(
                stream,
                cache,
                &ParserContext {
                    recovery_disabled: false,
                    ..context.clone()
                },
            )
        }
        &[] => parser_expr(rules, blocks, expr, rule_args, vars)
            .parse(stream, cache, context)
            .map(|pr| pr.rtrn),
    }
}
