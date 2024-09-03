use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::RuleExpr;
use crate::parser2::parse_sequence::ParserSequence;
use crate::parser2::{PResult, ParserChoiceSub, ParserState};
use crate::core::adaptive::BlockState;
use crate::parser2::add_rule::BlockCtx;

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    pub fn parse_expr(&mut self, expr: &'arn RuleExpr<'arn, 'grm>, blocks: BlockCtx<'arn, 'grm>) -> PResult<E> {
        match expr {
            RuleExpr::RunVar(rule_str, _) => {
                // Figure out which rule the variable `rule` refers to
                let Some(rule) = self.sequence_state.vars.get(rule_str) else {
                    panic!("Tried to run variable `{rule_str}` as a rule, but it was not defined.");
                };
                let Some(rule) = rule.as_rule_id() else {
                    panic!(
                        "Tried to run variable `{rule_str}` as a rule, but it did not refer to a rule."
                    );
                };
                self.add_rule(rule);
                Ok(())
            }
            RuleExpr::CharClass(cc) => self.parse_char(|c| cc.contains(*c)),
            RuleExpr::Literal(lit) => self.parse_chars(lit.chars()),
            RuleExpr::Repeat {
                expr,
                min,
                max,
                delim,
            } => {
                self.sequence_stack.push(ParserSequence::Repeat {
                    expr,
                    delim,
                    min: *min,
                    max: *max,
                    last_pos: None,
                    blocks
                });
                Ok(())
            }
            RuleExpr::Sequence(seqs) => {
                self.sequence_stack.push(ParserSequence::Exprs(seqs, blocks));
                Ok(())
            }
            RuleExpr::Choice(choices) => {
                let (first_choice, rest_choices) =
                    choices.split_first().expect("Choices not empty");

                if !rest_choices.is_empty() {
                    self.add_choice(ParserChoiceSub::Exprs(rest_choices, blocks));
                }
                self.add_expr(first_choice, blocks);
                Ok(())
            }
            RuleExpr::NameBind(_, expr) => {
                self.add_expr(expr, blocks);
                Ok(())
            }
            RuleExpr::Action(expr, _) => {
                self.add_expr(expr, blocks);
                Ok(())
            }
            RuleExpr::SliceInput(expr) => {
                self.add_expr(expr, blocks);
                Ok(())
            }
            RuleExpr::PosLookahead(expr) => {
                self.sequence_stack
                    .push(ParserSequence::PositiveLookaheadEnd {
                        sequence_state: self.sequence_state,
                    });
                self.add_expr(expr, blocks);
                Ok(())
            }
            RuleExpr::NegLookahead(expr) => {
                self.add_choice(ParserChoiceSub::NegativeLookaheadFail);
                self.sequence_stack
                    .push(ParserSequence::NegativeLookaheadEnd {
                        sequence_state: self.sequence_state,
                    });
                self.add_expr(expr, blocks);
                Ok(())
            }
            RuleExpr::This => {
                self.add_blocks(blocks);
                Ok(())
            },
            RuleExpr::Next => {
                assert!(blocks.len() > 1);
                self.add_blocks(&blocks[1..]);
                Ok(())
            },
            RuleExpr::AtAdapt(_, _) => todo!(),
            RuleExpr::Guid => Ok(()),
        }
    }
}
