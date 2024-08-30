use std::cmp::Ordering;
use std::slice;
use crate::core::adaptive::{Constructor, GrammarState, RuleId, RuleState};
use crate::core::cache::Allocs;
use crate::core::pos::Pos;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::{GrammarFile, RuleExpr};
use crate::parser2;
use crate::parser2::{ParserChoice, ParserChoiceSub, ParserSequence, ParserSequenceSub, ParserState, SeqState};

impl<'arn, 'grm: 'arn, E: ParseError<L= ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    pub fn fail(&mut self, e: E) -> Result<(), AggregatedParseError<'grm, E>> {
        self.add_error(e);

        'outer: while let Some(s) = self.choice_stack.last_mut() {
            self.sequence_state = s.sequence_state;
            match &mut s.choice {
                ParserChoiceSub::Blocks(bs, cs) => {
                    // Find the fist constructor from a block
                    let c = loop {
                        let Some(c) = take_first(cs) else {
                            let Some(b) = take_first(bs) else {
                                self.choice_stack.pop();
                                continue 'outer;
                            };
                            continue
                        };
                        break c;
                    };
                    self.add_constructor(c);
                }
                ParserChoiceSub::Exprs(exprs) => {
                    let Some(expr) = take_first(exprs) else {
                        self.choice_stack.pop();
                        continue 'outer;
                    };
                    self.add_expr(expr);
                }
            }
            return Ok(())
        }

        Err(self.completely_fail())
    }

    pub fn add_error(&mut self, e: E) {
        match &mut self.furthest_error {
            None => {
                self.furthest_error = Some((e, self.sequence_state.pos))
            }
            Some((cur_err, cur_pos)) => {
                match self.sequence_state.pos.cmp(cur_pos) {
                    Ordering::Less => {}
                    Ordering::Equal => {
                        *cur_err = cur_err.clone().merge(e)
                    }
                    Ordering::Greater => {
                        *cur_pos = self.sequence_state.pos;
                        *cur_err = e;
                    }
                }
            }
        }
    }

    pub fn completely_fail(&mut self) -> AggregatedParseError<'grm, E> {
        AggregatedParseError {
            input: self.input,
            errors: vec![self.furthest_error.take().expect("Cannot fail without error").0],
        }
    }
}

pub fn take_first<'a, T>(slice: &mut &'a [T]) -> Option<&'a T> {
    let (first, rem) = slice.split_first()?;
    *slice = rem;
    Some(first)
}