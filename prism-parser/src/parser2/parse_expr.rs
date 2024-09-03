use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::RuleExpr;
use crate::parser2::{
    PResult, ParserChoiceSub, ParserState
    ,
};
use std::slice;
use crate::parser2::parse_sequence::ParserSequence;
impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    pub fn parse_expr(&mut self, expr: &'arn RuleExpr<'arn, 'grm>) -> PResult<E> {
        match expr {
            RuleExpr::RunVar(rule_str, _) => {
                // Figure out which rule the variable `rule` refers to
                let Some(rule) = self.sequence_state.vars.get(rule_str) else {
                    panic!(
                        "Tried to run variable `{rule_str}` as a rule, but it was not defined."
                    );
                };
                let Some(rule) = rule.as_rule_id() else {
                    panic!(
                        "Tried to run variable `{rule_str}` as a rule, but it did not refer to a rule."
                    );
                };
                self.add_rule(rule);
                Ok(())
            },
            RuleExpr::CharClass(cc) => self.parse_char(|c| cc.contains(*c)),
            RuleExpr::Literal(lit) => self.parse_chars(lit.chars()),
            RuleExpr::Repeat { expr, min, max, delim } => {
                self.sequence_stack.push(ParserSequence::Repeat {
                    expr,
                    delim,
                    min: *min,
                    max: *max,
                    last_pos: None,
                });
                Ok(())
            },
            RuleExpr::Sequence(seqs) => {
                self.sequence_stack.push(ParserSequence::Exprs(seqs));
                Ok(())
            }
            RuleExpr::Choice(choices) => {
                let (first_choice, rest_choices) =
                    choices.split_first().expect("Choices not empty");

                if !rest_choices.is_empty() {
                    self.add_choice(ParserChoiceSub::Exprs(rest_choices));
                }
                self.sequence_stack.push(ParserSequence::Exprs(slice::from_ref(first_choice)));
                Ok(())
            }
            RuleExpr::NameBind(_, expr) => {
                self.add_expr(expr);
                Ok(())
            },
            RuleExpr::Action(expr, _) => {
                self.add_expr(expr);
                Ok(())
            },
            RuleExpr::SliceInput(expr) => {
                self.add_expr(expr);
                Ok(())
            },
            RuleExpr::PosLookahead(expr) => {
                self.sequence_stack.push(ParserSequence::PositiveLookaheadEnd {
                    sequence_state: self.sequence_state
                });
                self.add_expr(expr);
                Ok(())
            },
            RuleExpr::NegLookahead(expr) => {
                self.add_choice(ParserChoiceSub::NegativeLookaheadFail);
                self.sequence_stack.push(ParserSequence::NegativeLookaheadEnd {
                    sequence_state: self.sequence_state
                });
                self.add_expr(expr);
                Ok(())
            },
            RuleExpr::This => todo!(),
            RuleExpr::Next => todo!(),
            RuleExpr::AtAdapt(_, _) => todo!(),
            RuleExpr::Guid => {
                Ok(())
            },
        }
    }
}
