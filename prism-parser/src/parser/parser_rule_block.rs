use crate::core::presult::PResult;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use std::collections::HashMap;

use crate::core::adaptive::{BlockState, Constructor, GrammarState};

use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::state::ParserState;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::grammar::rule_annotation::RuleAnnotation;
use crate::grammar::rule_expr::RuleExpr;
use crate::parsable::parsed::Parsed;
use crate::parser::VarMap;

impl<'arn, 'grm: 'arn, Env, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, Env, E> {
    pub fn parse_rule_block(
        &mut self,
        rules: &'arn GrammarState<'arn, 'grm>,
        blocks: &'arn [BlockState<'arn, 'grm>],
        rule_args: VarMap<'arn, 'grm>,
        pos: Pos,
        context: ParserContext,
        penv: &mut Env,
        eval_ctx: Parsed<'arn, 'grm>,
    ) -> PResult<Parsed<'arn, 'grm>, E> {
        self.parse_cache_recurse(
            |state, pos| {
                state.parse_sub_blocks(rules, blocks, rule_args, pos, context, penv, eval_ctx)
            },
            blocks,
            rule_args,
            rules.unique_id(),
            pos,
            context,
        )
    }

    fn parse_sub_blocks(
        &mut self,
        rules: &'arn GrammarState<'arn, 'grm>,
        blocks: &'arn [BlockState<'arn, 'grm>],
        rule_args: VarMap<'arn, 'grm>,
        pos: Pos,
        context: ParserContext,
        penv: &mut Env,
        eval_ctx: Parsed<'arn, 'grm>,
    ) -> PResult<Parsed<'arn, 'grm>, E> {
        match blocks {
            [] => unreachable!(),
            [b] => self.parse_sub_constructors(
                rules,
                blocks,
                rule_args,
                b.constructors,
                pos,
                context,
                penv,
                eval_ctx,
            ),
            [b, brest @ ..] => {
                // Parse current
                let res = self.parse_sub_constructors(
                    rules,
                    blocks,
                    rule_args,
                    b.constructors,
                    pos,
                    context,
                    penv,
                    eval_ctx,
                );

                // Parse next with recursion check
                res.merge_choice_chain(|| {
                    self.parse_rule_block(rules, brest, rule_args, pos, context, penv, eval_ctx)
                })
            }
        }
    }

    fn parse_sub_constructors(
        &mut self,
        rules: &'arn GrammarState<'arn, 'grm>,
        blocks: &'arn [BlockState<'arn, 'grm>],
        rule_args: VarMap<'arn, 'grm>,
        es: &'arn [Constructor<'arn, 'grm>],
        pos: Pos,
        context: ParserContext,
        penv: &mut Env,
        eval_ctx: Parsed<'arn, 'grm>,
    ) -> PResult<Parsed<'arn, 'grm>, E> {
        match es {
            [] => PResult::new_err(E::new(pos), pos),
            [
                (AnnotatedRuleExpr { annotations, expr }, rule_ctx),
                rest @ ..,
            ] => {
                let rule_ctx = rule_ctx.into_iter();
                let rule_args_iter = rule_args.into_iter();
                let vars: VarMap<'arn, 'grm> =
                    VarMap::from_iter(rule_args_iter.chain(rule_ctx), self.alloc);

                self.parse_sub_annotations(
                    rules,
                    blocks,
                    rule_args,
                    annotations,
                    expr,
                    vars,
                    pos,
                    context,
                    penv,
                    eval_ctx,
                )
                .merge_choice_chain(|| {
                    self.parse_sub_constructors(
                        rules, blocks, rule_args, rest, pos, context, penv, eval_ctx,
                    )
                })
            }
        }
    }

    fn parse_sub_annotations(
        &mut self,
        rules: &'arn GrammarState<'arn, 'grm>,
        blocks: &'arn [BlockState<'arn, 'grm>],
        rule_args: VarMap<'arn, 'grm>,
        annots: &'arn [RuleAnnotation<'grm>],
        expr: &'arn RuleExpr<'arn, 'grm>,
        vars: VarMap<'arn, 'grm>,
        pos: Pos,
        context: ParserContext,
        penv: &mut Env,
        eval_ctx: Parsed<'arn, 'grm>,
    ) -> PResult<Parsed<'arn, 'grm>, E> {
        match annots {
            [RuleAnnotation::Error(err_label), rest @ ..] => {
                let mut res = self.parse_sub_annotations(
                    rules, blocks, rule_args, rest, expr, vars, pos, context, penv, eval_ctx,
                );
                res.add_label_explicit(ErrorLabel::Explicit(
                    pos.span_to(res.end_pos().next(self.input).0),
                    *err_label,
                ));
                res
            }
            [RuleAnnotation::DisableLayout, rest @ ..] => self.parse_with_layout(
                rules,
                vars,
                |state, pos, penv| {
                    state.parse_sub_annotations(
                        rules,
                        blocks,
                        rule_args,
                        rest,
                        expr,
                        vars,
                        pos,
                        ParserContext {
                            layout_disabled: true,
                            ..context
                        },
                        penv,
                        eval_ctx,
                    )
                },
                pos,
                context,
                penv,
            ),
            [RuleAnnotation::EnableLayout, rest @ ..] => self.parse_sub_annotations(
                rules,
                blocks,
                rule_args,
                rest,
                expr,
                vars,
                pos,
                ParserContext {
                    layout_disabled: false,
                    ..context
                },
                penv,
                eval_ctx,
            ),
            [
                RuleAnnotation::DisableRecovery | RuleAnnotation::EnableRecovery,
                rest @ ..,
            ] => self.parse_sub_annotations(
                rules,
                blocks,
                rule_args,
                rest,
                expr,
                vars,
                pos,
                ParserContext {
                    recovery_disabled: true,
                    ..context
                },
                penv,
                eval_ctx,
            ),
            &[] => self
                .parse_expr(
                    expr,
                    rules,
                    blocks,
                    rule_args,
                    vars,
                    pos,
                    context,
                    penv,
                    eval_ctx,
                    &mut HashMap::new(),
                )
                .map(|pr| pr.rtrn),
        }
    }
}
