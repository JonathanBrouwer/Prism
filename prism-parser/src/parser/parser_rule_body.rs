use crate::core::cache::parser_cache_recurse;
use crate::core::parser::Parser;
use crate::core::presult::PResult;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;

use crate::core::adaptive::{Constructor, GrammarState};
use crate::rule_action::RuleAction;
use by_address::ByAddress;

use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::recovery::recovery_point;
use crate::core::state::PState;
use crate::grammar::{RuleAnnotation, RuleExpr};
use crate::parser::parser_layout::parser_with_layout;
use crate::parser::parser_rule_expr::parser_expr;
use crate::parser::var_map::{BlockCtx, VarMap};
use crate::rule_action::action_result::ActionResult;

pub fn parser_body_cache_recurse<
    'a,
    'arn: 'a,
    'grm: 'arn,
    E: ParseError<L = ErrorLabel<'grm>> + 'grm,
>(
    rules: &'arn GrammarState<'arn, 'grm>,
    block_ctx: BlockCtx<'arn, 'grm>,
) -> impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E> + 'a {
    move |pos: Pos, state: &mut PState<'arn, 'grm, E>, context: ParserContext| {
        const RED_ZONE: usize = 1024 * 1024;
        stacker::maybe_grow(RED_ZONE, RED_ZONE * 64, || {
            parser_cache_recurse(
                &parser_body_sub_blocks(rules, block_ctx),
                block_ctx,
                rules.unique_id(),
            )
            .parse(pos, state, context)
        })
    }
}

fn parser_body_sub_blocks<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
    rules: &'arn GrammarState<'arn, 'grm>,
    (block_state, rule_args): BlockCtx<'arn, 'grm>,
) -> impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E> + 'a {
    move |pos: Pos,
          state: &mut PState<'arn, 'grm, E>,
          context: ParserContext|
          -> PResult<&'arn ActionResult<'arn, 'grm>, E> {
        match block_state.as_ref() {
            [] => unreachable!(),
            [b] => {
                parser_body_sub_constructors(rules, (block_state, rule_args), &b.constructors[..])
                    .parse(pos, state, context)
            }
            [b, brest @ ..] => {
                // Parse current
                let res = parser_body_sub_constructors(
                    rules,
                    (block_state, rule_args),
                    &b.constructors[..],
                )
                .parse(pos, state, context);

                // Parse next with recursion check
                res.merge_choice_parser(
                    &parser_body_cache_recurse(rules, (brest, rule_args)),
                    pos,
                    state,
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
    (block_state, rule_args): BlockCtx<'arn, 'grm>,
    es: &'arn [Constructor<'arn, 'grm>],
) -> impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E> + 'a {
    move |pos: Pos, state: &mut PState<'arn, 'grm, E>, context: ParserContext| match es {
        [] => PResult::new_err(E::new(pos.span_to(pos)), pos),
        [(crate::grammar::AnnotatedRuleExpr(annots, expr), rule_ctx), rest @ ..] => {
            let rule_ctx = rule_ctx.iter_cloned();
            let rule_args_iter = rule_args.iter_cloned();
            let vars: VarMap<'arn, 'grm> =
                VarMap::from_iter(rule_args_iter.chain(rule_ctx), state.alloc);

            let res =
                parser_body_sub_annotations(rules, (block_state, rule_args), annots, expr, vars)
                    .parse(pos, state, context)
                    .merge_choice_parser(
                        &parser_body_sub_constructors(rules, (block_state, rule_args), rest),
                        pos,
                        state,
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
    block_state: BlockCtx<'arn, 'grm>,
    annots: &'arn [RuleAnnotation<'grm>],
    expr: &'arn RuleExpr<'grm, RuleAction<'arn, 'grm>>,
    vars: VarMap<'arn, 'grm>,
) -> impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E> + 'a {
    move |pos: Pos, state: &mut PState<'arn, 'grm, E>, context: ParserContext| match annots {
        [RuleAnnotation::Error(err_label), rest @ ..] => {
            let mut res = parser_body_sub_annotations(rules, block_state, rest, expr, vars)
                .parse(pos, state, context);
            res.add_label_explicit(ErrorLabel::Explicit(
                pos.span_to(res.end_pos().next(state.input).0),
                *err_label,
            ));
            res
        }
        [RuleAnnotation::DisableLayout, rest @ ..] => {
            parser_with_layout(rules, vars, &move |pos: Pos,
                                                   state: &mut PState<'arn, 'grm, E>,
                                                   context: ParserContext|
                  -> PResult<_, E> {
                parser_body_sub_annotations(rules, block_state, rest, expr, vars).parse(
                    pos,
                    state,
                    ParserContext {
                        layout_disabled: true,
                        ..context
                    },
                )
            })
            .parse(pos, state, context)
        }
        [RuleAnnotation::EnableLayout, rest @ ..] => {
            parser_with_layout(rules, vars, &move |pos: Pos,
                                                   state: &mut PState<'arn, 'grm, E>,
                                                   context: ParserContext|
                  -> PResult<_, E> {
                parser_body_sub_annotations(rules, block_state, rest, expr, vars).parse(
                    pos,
                    state,
                    ParserContext {
                        layout_disabled: false,
                        ..context
                    },
                )
            })
            .parse(pos, state, context)
        }
        [RuleAnnotation::DisableRecovery, rest @ ..] => recovery_point(
            move |pos: Pos, state: &mut PState<'arn, 'grm, E>, context: ParserContext| {
                parser_body_sub_annotations(rules, block_state, rest, expr, vars).parse(
                    pos,
                    state,
                    ParserContext {
                        recovery_disabled: true,
                        ..context
                    },
                )
            },
        )
        .parse(pos, state, context),
        [RuleAnnotation::EnableRecovery, rest @ ..] => {
            parser_body_sub_annotations(rules, block_state, rest, expr, vars).parse(
                pos,
                state,
                ParserContext {
                    recovery_disabled: false,
                    ..context
                },
            )
        }
        &[] => parser_expr(rules, block_state, expr, vars)
            .parse(pos, state, context)
            .map(|pr| pr.rtrn),
    }
}
