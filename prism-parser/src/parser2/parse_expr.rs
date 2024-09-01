use crate::core::adaptive::{GrammarState, RuleId, RuleState};
use crate::core::cache::Allocs;
use crate::core::pos::Pos;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::{GrammarFile, RuleExpr};
use crate::parser2;
use crate::parser2::{
    PResult, ParserChoice, ParserChoiceSub, ParserSequence, ParserState,
    SeqState,
};
use std::slice;
use crate::parser::var_map::VarMapValue;

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
            RuleExpr::Repeat { expr, min, max, delim } => todo!(),
            RuleExpr::Sequence(seqs) => {
                self.sequence_stack.push(ParserSequence::Exprs(seqs));
                Ok(())
            }
            RuleExpr::Choice(choices) => {
                let (first_choice, rest_choices) =
                    choices.split_first().expect("Choices not empty");

                self.add_choice(ParserChoiceSub::Exprs(rest_choices));
                self.sequence_stack.push(ParserSequence::Exprs(slice::from_ref(first_choice)));
                Ok(())
            }
            RuleExpr::NameBind(_, expr) => {
                self.sequence_stack.push(ParserSequence::Exprs(slice::from_ref(expr)));
                Ok(())
            },
            RuleExpr::Action(expr, _) => {
                self.sequence_stack.push(ParserSequence::Exprs(slice::from_ref(expr)));
                Ok(())
            },
            RuleExpr::SliceInput(expr) => {
                self.sequence_stack.push(ParserSequence::Exprs(slice::from_ref(expr)));
                Ok(())
            },
            RuleExpr::PosLookahead(_) => todo!(),
            RuleExpr::NegLookahead(_) => todo!(),
            RuleExpr::This => todo!(),
            RuleExpr::Next => todo!(),
            RuleExpr::AtAdapt(_, _) => todo!(),
            RuleExpr::Guid => {
                Ok(())
            },
        }
    }
}
