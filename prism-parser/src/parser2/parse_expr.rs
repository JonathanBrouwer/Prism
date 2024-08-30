use crate::core::adaptive::{GrammarState, RuleId, RuleState};
use crate::core::cache::Allocs;
use crate::core::pos::Pos;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::{GrammarFile, RuleExpr};
use crate::parser2;
use crate::parser2::{
    PResult, ParserChoice, ParserChoiceSub, ParserSequence, ParserSequenceSub, ParserState,
    SeqState,
};
use std::slice;

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    pub fn parse_expr(&mut self, expr: &'arn RuleExpr<'arn, 'grm>) -> PResult<E> {
        match expr {
            RuleExpr::RunVar(_, var) => {
                todo!()
            },
            RuleExpr::CharClass(cc) => self.parse_char(|c| cc.contains(*c)),
            RuleExpr::Literal(lit) => self.parse_chars(lit.chars()),
            RuleExpr::Repeat { .. } => todo!(),
            RuleExpr::Sequence(seqs) => {
                self.sequence_stack.push(ParserSequence {
                    sequence: ParserSequenceSub::Exprs(seqs),
                });
                Ok(())
            }
            RuleExpr::Choice(choices) => {
                let (first_choice, rest_choices) =
                    choices.split_first().expect("Choices not empty");

                self.choice_stack.push(ParserChoice {
                    choice: ParserChoiceSub::Exprs(rest_choices),
                    sequence_state: self.sequence_state,
                });
                self.sequence_stack.push(ParserSequence {
                    sequence: ParserSequenceSub::Exprs(slice::from_ref(first_choice)),
                });
                Ok(())
            }
            RuleExpr::NameBind(_, expr) => {
                self.sequence_stack.push(ParserSequence {
                    sequence: ParserSequenceSub::Exprs(slice::from_ref(expr)),
                });
                Ok(())
            },
            RuleExpr::Action(expr, _) => {
                self.sequence_stack.push(ParserSequence {
                    sequence: ParserSequenceSub::Exprs(slice::from_ref(expr)),
                });
                Ok(())
            },
            RuleExpr::SliceInput(expr) => {
                self.sequence_stack.push(ParserSequence {
                    sequence: ParserSequenceSub::Exprs(slice::from_ref(expr)),
                });
                Ok(())
            },
            RuleExpr::PosLookahead(_) => todo!(),
            RuleExpr::NegLookahead(_) => todo!(),
            RuleExpr::This => todo!(),
            RuleExpr::Next => todo!(),
            RuleExpr::AtAdapt(_, _) => todo!(),
            RuleExpr::Guid => todo!(),
        }
    }
}
