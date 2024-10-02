use crate::core::cache::parser_cache_recurse;
use crate::core::parser::Parser;
use crate::core::presult::PResult;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;

use crate::core::adaptive::{Constructor, GrammarState};

use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::state::ParserState;
use crate::grammar::action_result::ActionResult;
use crate::grammar::{RuleAnnotation, RuleExpr};
use crate::parser::var_map::{BlockCtx, VarMap};

pub fn parser_body_cache_recurse<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>>(
    rules: &'arn GrammarState<'arn, 'grm>,
    block_ctx: BlockCtx<'arn, 'grm>,
) -> impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E> + 'a {
    move |pos: Pos, state: &mut ParserState<'arn, 'grm, E>, context: ParserContext| {
        parser_cache_recurse(
            &parser_body_sub_blocks(rules, block_ctx),
            block_ctx,
            rules.unique_id(),
        )
        .parse(pos, state, context)
    }
}

fn parser_body_sub_blocks<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>>(
    rules: &'arn GrammarState<'arn, 'grm>,
    (block_state, rule_args): BlockCtx<'arn, 'grm>,
) -> impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E> + 'a {
    move |pos: Pos,
          state: &mut ParserState<'arn, 'grm, E>,
          context: ParserContext|
          -> PResult<&'arn ActionResult<'arn, 'grm>, E> {
        match block_state {
            [] => unreachable!(),
            [b] => parser_body_sub_constructors(rules, (block_state, rule_args), b.constructors)
                .parse(pos, state, context),
            [b, brest @ ..] => {
                // Parse current
                let res =
                    parser_body_sub_constructors(rules, (block_state, rule_args), b.constructors)
                        .parse(pos, state, context);

                // Parse next with recursion check
                res.merge_choice_chain(|| {
                    parser_body_cache_recurse(rules, (brest, rule_args)).parse(pos, state, context)
                })
            }
        }
    }
}

fn parser_body_sub_constructors<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>>(
    rules: &'arn GrammarState<'arn, 'grm>,
    (block_state, rule_args): BlockCtx<'arn, 'grm>,
    es: &'arn [Constructor<'arn, 'grm>],
) -> impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E> + 'a {
    move |pos: Pos, state: &mut ParserState<'arn, 'grm, E>, context: ParserContext| match es {
        [] => PResult::new_err(E::new(pos.span_to(pos)), pos),
        [(crate::grammar::AnnotatedRuleExpr(annots, expr), rule_ctx), rest @ ..] => {
            let rule_ctx = rule_ctx.iter_cloned();
            let rule_args_iter = rule_args.iter_cloned();
            let vars: VarMap<'arn, 'grm> =
                VarMap::from_iter(rule_args_iter.chain(rule_ctx), state.alloc);

            let res =
                parser_body_sub_annotations(rules, (block_state, rule_args), annots, expr, vars)
                    .parse(pos, state, context)
                    .merge_choice_chain(|| {
                        parser_body_sub_constructors(rules, (block_state, rule_args), rest)
                            .parse(pos, state, context)
                    });
            res
        }
    }
}

fn parser_body_sub_annotations<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>>(
    rules: &'arn GrammarState<'arn, 'grm>,
    block_state: BlockCtx<'arn, 'grm>,
    annots: &'arn [RuleAnnotation<'grm>],
    expr: &'arn RuleExpr<'arn, 'grm>,
    vars: VarMap<'arn, 'grm>,
) -> impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E> + 'a {
    move |pos: Pos, state: &mut ParserState<'arn, 'grm, E>, context: ParserContext| match annots {
        [RuleAnnotation::Error(err_label), rest @ ..] => {
            let mut res = parser_body_sub_annotations(rules, block_state, rest, expr, vars)
                .parse(pos, state, context);
            res.add_label_explicit(ErrorLabel::Explicit(
                pos.span_to(res.end_pos().next(state.input).0),
                *err_label,
            ));
            res
        }
        [RuleAnnotation::DisableLayout, rest @ ..] => state.parse_with_layout(
            rules,
            vars,
            |state, pos| {
                parser_body_sub_annotations(rules, block_state, rest, expr, vars).parse(
                    pos,
                    state,
                    ParserContext {
                        layout_disabled: true,
                        ..context
                    },
                )
            },
            pos,
            context,
        ),
        [RuleAnnotation::EnableLayout, rest @ ..] => {
            parser_body_sub_annotations(rules, block_state, rest, expr, vars).parse(
                pos,
                state,
                ParserContext {
                    layout_disabled: false,
                    ..context
                },
            )
        }
        [RuleAnnotation::DisableRecovery | RuleAnnotation::EnableRecovery, rest @ ..] => {
            parser_body_sub_annotations(rules, block_state, rest, expr, vars).parse(
                pos,
                state,
                ParserContext {
                    recovery_disabled: true,
                    ..context
                },
            )
        }
        &[] => state
            .parse_expr(rules, block_state, expr, vars, pos, context)
            .map(|pr| pr.rtrn),
    }
}
