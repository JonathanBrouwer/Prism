use crate::core::adaptive::{BlockState, Constructor, GrammarState};
use crate::core::arc_ref::BorrowedArcSlice;
use crate::core::context::{PV, ParserContext};
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use crate::grammar::rule_annotation::RuleAnnotation;
use crate::grammar::rule_expr::RuleExpr;
use crate::parsable::parsed::Parsed;
use crate::parser::VarMap;
use std::collections::HashMap;
use std::sync::Arc;

impl<Db, E: ParseError<L = ErrorLabel>> ParserState<Db, E> {
    pub fn parse_rule_block(
        &mut self,
        rules: &GrammarState,
        blocks: BorrowedArcSlice<Arc<BlockState>>,
        rule_args: &VarMap,
        pos: Pos,
        context: ParserContext,
        penv: &mut Db,
        eval_ctx: &Parsed,
    ) -> PResult<PV, E> {
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
        rules: &GrammarState,
        blocks: BorrowedArcSlice<Arc<BlockState>>,
        rule_args: &VarMap,
        pos: Pos,
        context: ParserContext,
        penv: &mut Db,
        eval_ctx: &Parsed,
    ) -> PResult<PV, E> {
        match &*blocks {
            [] => unreachable!(),
            [b] => self.parse_sub_constructors(
                rules,
                blocks,
                rule_args,
                &b.constructors,
                pos,
                context,
                penv,
                eval_ctx,
            ),
            [b, ..] => {
                // Parse current
                let res = self.parse_sub_constructors(
                    rules,
                    blocks,
                    rule_args,
                    &b.constructors,
                    pos,
                    context,
                    penv,
                    eval_ctx,
                );

                // Parse next with recursion check
                res.merge_choice_chain(|| {
                    self.parse_rule_block(
                        rules,
                        blocks.slice(1..),
                        rule_args,
                        pos,
                        context,
                        penv,
                        eval_ctx,
                    )
                })
            }
        }
    }

    fn parse_sub_constructors(
        &mut self,
        rules: &GrammarState,
        blocks: BorrowedArcSlice<Arc<BlockState>>,
        rule_args: &VarMap,
        es: &[Constructor],
        pos: Pos,
        context: ParserContext,
        penv: &mut Db,
        eval_ctx: &Parsed,
    ) -> PResult<PV, E> {
        match es.split_first() {
            None => PResult::new_err(E::new(pos), pos),
            Some(((expr, rule_ctx), rest)) => {
                let vars: VarMap = rule_args.extend(rule_ctx.iter_cloned());

                self.parse_sub_annotations(
                    rules,
                    blocks,
                    rule_args,
                    &expr.annotations,
                    &expr.expr,
                    &vars,
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
        rules: &GrammarState,
        blocks: BorrowedArcSlice<Arc<BlockState>>,
        rule_args: &VarMap,
        annots: &[Arc<RuleAnnotation>],
        expr: &RuleExpr,
        vars: &VarMap,
        pos: Pos,
        context: ParserContext,
        penv: &mut Db,
        eval_ctx: &Parsed,
    ) -> PResult<PV, E> {
        match annots.split_first() {
            Some((annot, rest)) => match &**annot {
                RuleAnnotation::Error(err_label) => {
                    let mut res = self.parse_sub_annotations(
                        rules, blocks, rule_args, rest, expr, vars, pos, context, penv, eval_ctx,
                    );
                    res.add_label_explicit(ErrorLabel::Explicit(
                        pos.span_to(res.end_pos().next(&self.input).0),
                        err_label.to_string(),
                    ));
                    res
                }
                RuleAnnotation::DisableLayout => self.parse_with_layout(
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
                RuleAnnotation::EnableLayout => self.parse_sub_annotations(
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
                RuleAnnotation::DisableRecovery | RuleAnnotation::EnableRecovery => self
                    .parse_sub_annotations(
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
                RuleAnnotation::Token(token) => {
                    // Parse recursively
                    let mut res = self.parse_with_layout(
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
                                    recovery_disabled: true,
                                    ..context
                                },
                                penv,
                                eval_ctx,
                            )
                        },
                        pos,
                        context,
                        penv,
                    );

                    // Add token information
                    res = res.map_with_span(|pv, span| PV::new_single(pv.parsed, *token, span));

                    // Add error label
                    res.add_label_explicit(ErrorLabel::Explicit(
                        pos.span_to(res.end_pos().next(&self.input).0),
                        token.to_string(),
                    ));

                    res
                }
            },
            None => self
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
