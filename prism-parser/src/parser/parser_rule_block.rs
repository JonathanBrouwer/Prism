use crate::core::presult::PResult;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;

use crate::core::adaptive::{Constructor, GrammarState};

use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::state::ParserState;
use crate::grammar::RuleExpr;
use crate::grammar::rule_annotation::RuleAnnotation;
use crate::parsable::parsed::Parsed;
use crate::parser::var_map::{BlockCtx, VarMap};

impl<'arn, 'grm, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    pub fn parse_rule_block(
        &mut self,
        rules: &'arn GrammarState<'arn, 'grm>,
        block_ctx: BlockCtx<'arn, 'grm>,
        pos: Pos,
        context: ParserContext,
    ) -> PResult<Parsed<'arn, 'grm>, E> {
        self.parse_cache_recurse(
            |state, pos| state.parse_sub_blocks(rules, block_ctx, pos, context),
            block_ctx,
            rules.unique_id(),
            pos,
            context,
        )
    }

    fn parse_sub_blocks(
        &mut self,
        rules: &'arn GrammarState<'arn, 'grm>,
        (block_state, rule_args): BlockCtx<'arn, 'grm>,
        pos: Pos,
        context: ParserContext,
    ) -> PResult<Parsed<'arn, 'grm>, E> {
        match block_state {
            [] => unreachable!(),
            [b] => self.parse_sub_constructors(
                rules,
                (block_state, rule_args),
                b.constructors,
                pos,
                context,
            ),
            [b, brest @ ..] => {
                // Parse current
                let res = self.parse_sub_constructors(
                    rules,
                    (block_state, rule_args),
                    b.constructors,
                    pos,
                    context,
                );

                // Parse next with recursion check
                res.merge_choice_chain(|| {
                    self.parse_rule_block(rules, (brest, rule_args), pos, context)
                })
            }
        }
    }

    fn parse_sub_constructors(
        &mut self,
        rules: &'arn GrammarState<'arn, 'grm>,
        (block_state, rule_args): BlockCtx<'arn, 'grm>,
        es: &'arn [Constructor<'arn, 'grm>],
        pos: Pos,
        context: ParserContext,
    ) -> PResult<Parsed<'arn, 'grm>, E> {
        match es {
            [] => PResult::new_err(E::new(pos.span_to(pos)), pos),
            [(crate::grammar::AnnotatedRuleExpr(annots, expr), rule_ctx), rest @ ..] => {
                let rule_ctx = rule_ctx.iter_cloned();
                let rule_args_iter = rule_args.iter_cloned();
                let vars: VarMap<'arn, 'grm> =
                    VarMap::from_iter(rule_args_iter.chain(rule_ctx), self.alloc);

                let res = self
                    .parse_sub_annotations(
                        rules,
                        (block_state, rule_args),
                        annots,
                        expr,
                        vars,
                        pos,
                        context,
                    )
                    .merge_choice_chain(|| {
                        self.parse_sub_constructors(
                            rules,
                            (block_state, rule_args),
                            rest,
                            pos,
                            context,
                        )
                    });
                res
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn parse_sub_annotations(
        &mut self,
        rules: &'arn GrammarState<'arn, 'grm>,
        block_state: BlockCtx<'arn, 'grm>,
        annots: &'arn [RuleAnnotation<'grm>],
        expr: &'arn RuleExpr<'arn, 'grm>,
        vars: VarMap<'arn, 'grm>,
        pos: Pos,
        context: ParserContext,
    ) -> PResult<Parsed<'arn, 'grm>, E> {
        match annots {
            [RuleAnnotation::Error(err_label), rest @ ..] => {
                let mut res =
                    self.parse_sub_annotations(rules, block_state, rest, expr, vars, pos, context);
                res.add_label_explicit(ErrorLabel::Explicit(
                    pos.span_to(res.end_pos().next(self.input).0),
                    *err_label,
                ));
                res
            }
            [RuleAnnotation::DisableLayout, rest @ ..] => self.parse_with_layout(
                rules,
                vars,
                |state, pos| {
                    state.parse_sub_annotations(
                        rules,
                        block_state,
                        rest,
                        expr,
                        vars,
                        pos,
                        ParserContext {
                            layout_disabled: true,
                            ..context
                        },
                    )
                },
                pos,
                context,
            ),
            [RuleAnnotation::EnableLayout, rest @ ..] => self.parse_sub_annotations(
                rules,
                block_state,
                rest,
                expr,
                vars,
                pos,
                ParserContext {
                    layout_disabled: false,
                    ..context
                },
            ),
            [RuleAnnotation::DisableRecovery | RuleAnnotation::EnableRecovery, rest @ ..] => self
                .parse_sub_annotations(
                    rules,
                    block_state,
                    rest,
                    expr,
                    vars,
                    pos,
                    ParserContext {
                        recovery_disabled: true,
                        ..context
                    },
                ),
            &[] => self
                .parse_expr(rules, block_state, expr, vars, pos, context)
                .map(|pr| pr.rtrn),
        }
    }
}
