use std::collections::HashMap;
use std::mem;
use crate::action::{ActionVisitor, IgnoreVisitor};
use crate::action::action_result::ActionResult;
use crate::core::adaptive::GrammarState;
use crate::core::context::{ParserContext};
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::action::apply_action::{apply_action};
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::from_action_result::parse_grammarfile;
use crate::grammar::rule_action::RuleAction;
use crate::grammar::{GrammarFile, RuleExpr};
use crate::parser::var_map::{BlockCtx, CapturedExpr, VarMap, VarMapValue};

impl<'arn, 'grm, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    pub fn parse_expr(
        &mut self,
        rules: &'arn GrammarState<'arn, 'grm>,
        block_ctx: BlockCtx<'arn, 'grm>,
        expr: &'arn RuleExpr<'arn, 'grm>,
        vars: VarMap<'arn, 'grm>,
        pos: Pos,
        context: ParserContext,
        visitor: &mut dyn ActionVisitor<'arn, 'grm>,
        free_visitors: &mut HashMap<&'grm str, &mut dyn ActionVisitor<'arn, 'grm>>
    ) -> PResult<(), E> {
        match expr {
            RuleExpr::RunVar(mut rule_str, args) => {
                let mut result_args = vec![];
                let mut args = args;
                let mut block_ctx = block_ctx;
                let mut vars = vars;

                loop {
                    // Figure out which rule the variable `rule` refers to
                    let Some(rule) = vars.get(rule_str) else {
                        panic!(
                            "Tried to run variable `{rule_str}` as a rule, but it was not defined."
                        );
                    };

                    result_args.splice(
                        0..0,
                        args.iter().map(|arg| {
                            VarMapValue::Expr(CapturedExpr {
                                expr: arg,
                                block_ctx,
                                vars,
                            })
                        }),
                    );

                    return match rule {
                        VarMapValue::Expr(captured) => {
                            // If the `Expr` we call is a rule, we might be using it as a higher-order rule
                            // We process this rule in a loop, using the context of the captured expression
                            if let RuleExpr::RunVar(sub_rule, sub_args) = captured.expr {
                                rule_str = sub_rule;
                                args = sub_args;
                                block_ctx = captured.block_ctx;
                                vars = captured.vars;
                                continue;
                            } else {
                                assert_eq!(
                                    result_args.len(),
                                    0,
                                    "Tried to apply an argument to a non-rule expr"
                                );
                                self.parse_expr(
                                    rules,
                                    captured.block_ctx,
                                    captured.expr,
                                    captured.vars,
                                    pos,
                                    context,
                                    visitor,
                                    &mut HashMap::new(),
                                )
                            }
                        }
                        VarMapValue::Value(value) => {
                            if let ActionResult::RuleId(rule) = value {
                                self.parse_rule(rules, *rule, &result_args, pos, context, visitor)
                            } else {
                                todo!()
                                //TODO remove this code and replace with $value expressions
                                // PResult::new_empty(PR::with_rtrn(value), pos)
                            }
                        }
                    };
                }
            }
            RuleExpr::CharClass(cc) => {
                self
                    .parse_with_layout(
                        rules,
                        vars,
                        |state, pos| state.parse_char(|c| cc.contains(*c), pos),
                        pos,
                        context,
                    )
                    .map(|(span, _)| visitor.visit_input_str(&self.input[span], span, self.allocs))
            },
            RuleExpr::Literal(literal) => self.parse_with_layout(
                rules,
                vars,
                |state, start_pos| {
                    let mut res = PResult::new_empty((), start_pos);
                    for char in literal.chars() {
                        let new_res = state.parse_char(|c| *c == char, res.end_pos());
                        res = res.merge_seq(new_res).map(|_| ());
                    }
                    let span = start_pos.span_to(res.end_pos());
                    let mut res =
                        res.map(|_| visitor.visit_input_str(&state.input[span], span, state.allocs));
                    res.add_label_implicit(ErrorLabel::Literal(span, *literal));
                    res
                },
                pos,
                context,
            ),
            RuleExpr::Repeat {
                expr,
                min,
                max,
                delim,
            } => {
                let mut res: PResult<(), E> = PResult::new_empty((), pos);
                let mut current_visitor = visitor;

                for i in 0..max.unwrap_or(u64::MAX) {
                    let current_visitor_ptr = current_visitor as *mut dyn ActionVisitor<'arn, 'grm>;
                    let mut next_visitors = current_visitor.visit_construct("Cons", 2, self.allocs).into_iter();
                    let element_visitor = next_visitors.next().unwrap();
                    let rest_visitor = next_visitors.next().unwrap();
                    assert!(next_visitors.next().is_none());

                    let pos = res.end_pos();
                    let part = if i == 0 {
                        self.parse_expr(rules, block_ctx, expr, vars, pos, context, element_visitor, &mut HashMap::new())
                    } else {
                        self.parse_expr(rules, block_ctx, delim, vars, pos, context, &mut IgnoreVisitor, &mut HashMap::new())
                            .merge_seq_chain(|pos| {
                                self.parse_expr(rules, block_ctx, expr, vars, pos, context, element_visitor, &mut HashMap::new())
                            })
                            .map(|((), ())| ())
                    };
                    let should_continue = part.is_ok();

                    if i < *min {
                        res = res.merge_seq(part).map(|((), ())| ());
                    } else {
                        res = res.merge_seq_opt(part).map(|((), _)| ());
                    };

                    if !should_continue {
                        // Safety: The mutable reference to current_visitor should be safe to use again here
                        unsafe { &mut *current_visitor_ptr }.visit_construct("Nil", 0, self.allocs);
                        return res;
                    } else {
                        current_visitor = rest_visitor;
                    }

                    // If the result is OK and the last pos has not changed, we got into an infinite loop
                    // We break out with an infinite loop error
                    // The i != 0 check is to make sure to take the delim into account
                    if i != 0 && res.end_pos() <= pos {
                        let span = pos.span_to(pos);
                        let mut e = E::new(span);
                        e.add_label_explicit(ErrorLabel::Debug(span, "INFLOOP"));
                        return PResult::new_err(e, pos);
                    }
                }

                current_visitor.visit_construct("Nil", 0, self.allocs);
                res
            }
            RuleExpr::Sequence(subs) => {
                let mut res = PResult::new_empty((), pos);
                for sub in *subs {
                    res = res
                        .merge_seq_chain(|pos| {
                            self.parse_expr(rules, block_ctx, sub, vars, pos, context, &mut IgnoreVisitor, free_visitors)
                        }).map(|((), ())| ());
                    if res.is_err() {
                        break
                    }
                }
                res
            }
            RuleExpr::Choice(subs) => {
                let mut res: PResult<(), E> = PResult::PErr(E::new(pos.span_to(pos)), pos);
                for sub in *subs {
                    res = res.merge_choice_chain(|| {
                        self.parse_expr(rules, block_ctx, sub, vars, pos, context, visitor, &mut HashMap::new())
                    });
                    if res.is_ok() {
                        break;
                    }
                }
                res
            }
            RuleExpr::NameBind(name, sub) => {
                let visitor: &mut _ = *free_visitors.get_mut(name).unwrap();
                self.parse_expr(rules, block_ctx, sub, vars, pos, context, visitor, &mut HashMap::new())
            }
            RuleExpr::Action(sub, action) => {
                let mut new_visit_map = apply_action(action, visitor, self.allocs);
                self.parse_expr(rules, block_ctx, sub, vars, pos, context, &mut IgnoreVisitor, &mut new_visit_map)
            }
            RuleExpr::SliceInput(sub) => {
                let res = self.parse_expr(rules, block_ctx, sub, vars, pos, context, &mut IgnoreVisitor, &mut HashMap::new());
                res.map_with_span(|_, span| {
                    visitor.visit_input_str(&self.input[span], span, self.allocs)
                })
            }
            RuleExpr::This => self
                .parse_rule_block(rules, block_ctx, pos, context, visitor),
            RuleExpr::Next => self
                .parse_rule_block(rules, (&block_ctx.0[1..], block_ctx.1), pos, context, visitor),
            RuleExpr::PosLookahead(sub) => self
                .parse_expr(rules, block_ctx, sub, vars, pos, context, &mut IgnoreVisitor, free_visitors)
                .positive_lookahead(pos),
            RuleExpr::NegLookahead(sub) => self
                .parse_expr(rules, block_ctx, sub, vars, pos, context, &mut IgnoreVisitor, &mut HashMap::new())
                .negative_lookahead(pos),
            RuleExpr::AtAdapt(ga, adapt_rule) => {
                todo!()
            //     // First, get the grammar actionresult
            //     let gr = if let Some(ar) = vars.get(ga) {
            //         if let VarMapValue::Value(v) = ar {
            //             v
            //         } else {
            //             panic!("")
            //         }
            //     } else {
            //         panic!("Name '{ga}' not in context")
            //     };
            //
            //     // Parse it into a grammar
            //     //TODO performance: We should have a cache for grammar files
            //     //TODO and grammar state + new grammar -> grammar state
            //     let g = match parse_grammarfile(gr, self.input, self.alloc, |ar, _| {
            //         Some(RuleAction::ActionResult(ar))
            //     }) {
            //         Some(g) => g,
            //         None => {
            //             let mut e = E::new(pos.span_to(pos));
            //             e.add_label_implicit(ErrorLabel::Explicit(
            //                 pos.span_to(pos),
            //                 EscapedString::from_escaped(
            //                     "language grammar to be correct, but adaptation AST was malformed.",
            //                 ),
            //             ));
            //             return PResult::new_err(e, pos);
            //         }
            //     };
            //     let g: &'arn GrammarFile<'arn, 'grm> = self.alloc.alloc(g);
            //
            //     // Create new grammarstate
            //     let (rules, _) = match rules.adapt_with(g, vars, Some(pos), self.alloc) {
            //         Ok(rules) => rules,
            //         Err(_) => {
            //             let mut e = E::new(pos.span_to(pos));
            //             e.add_label_implicit(ErrorLabel::Explicit(
            //                 pos.span_to(pos),
            //                 EscapedString::from_escaped(
            //                     "language grammar to be correct, but adaptation created cycle in block order.",
            //                 ),
            //             ));
            //             return PResult::new_err(e, pos);
            //         }
            //     };
            //     let rules: &'arn GrammarState = self.alloc.alloc(rules);
            //
            //     let rule = vars
            //         .get(adapt_rule)
            //         .or_else(|| vars.get(adapt_rule))
            //         .unwrap()
            //         .as_rule_id()
            //         .expect("Adaptation rule exists");
            //
            //     // Parse body
            //     let mut res = self
            //         .parse_rule(rules, rule, &[], pos, context)
            //         .map(PR::with_rtrn);
            //     res.add_label_implicit(ErrorLabel::Debug(pos.span_to(pos), "adaptation"));
            //     res
            }
            RuleExpr::Guid => {
                visitor.visit_guid(self.guid_counter, self.allocs);
                self.guid_counter += 1;
                PResult::new_empty(
                    (),
                    pos,
                )
            }
        }
    }
}

pub fn take<T, F>(mut_ref: &mut T, closure: impl FnOnce(T) -> T) {
    use std::ptr;

    unsafe {
        let old_t = ptr::read(mut_ref);
        let new_t = closure(old_t);
        ptr::write(mut_ref, new_t);
    }
}
