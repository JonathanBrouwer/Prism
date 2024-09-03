use crate::core::adaptive::BlockState;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::RuleExpr;
use crate::parser2::{ParserChoiceSub, ParserState, SequenceState};
use crate::parser2::cache::CacheKey;
use crate::parser2::fail::take_first;

impl<'arn, 'grm: 'arn, E: ParseError<L= ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    /// Parse the top-most sequence from the sequence stack
    /// Returns an error if an unrecoverable error occurred
    pub fn parse_sequence(&mut self) -> Result<(), AggregatedParseError<'grm, E>> {
        let s = self.sequence_stack.last_mut().expect("Precondition of method");
        match s {
            ParserSequence::Exprs(exprs) => {
                //TODO use stdlib when slice::take_first stabilizes
                let Some(expr) = take_first(exprs) else {
                    self.sequence_stack.pop().unwrap();
                    return Ok(());
                };

                match self.parse_expr(expr) {
                    Ok(()) => {}
                    Err(e) => {
                        let () = self.fail(e)?;
                    }
                }
            }
            ParserSequence::PopChoice(..) => {
                self.drop_choice();
            }
            ParserSequence::Block(block) => {
                let key = CacheKey::new(
                    self.sequence_state.pos,
                    block,
                    self.sequence_state.grammar_state
                );
                if let Some(cached_result) = self.cache.get(&key) {
                    match cached_result {
                        Ok(new_sequence_state) => {
                            self.sequence_stack.pop().unwrap();
                            self.sequence_state = *new_sequence_state;
                        }
                        Err(e) => {
                            let e = e.clone();
                            let () = self.fail(e)?;
                        }
                    }
                    return Ok(())
                }

                // Add initial error seed to prevent left recursion
                self.cache.insert(key.clone(), Err(E::new(self.sequence_state.pos.span_to(self.sequence_state.pos))));

                // Add constructors of this block
                let (first_constructor, rest_constructors) = block.constructors.split_first().expect("Block not empty");
                self.sequence_stack.pop().unwrap();
                self.sequence_stack.push(ParserSequence::CacheOk {
                    key
                });
                if !rest_constructors.is_empty() {
                    self.add_choice(ParserChoiceSub::Constructors(rest_constructors));
                }
                self.add_constructor(first_constructor);
            }
            ParserSequence::CacheOk { key } => {
                self.cache.insert(key.clone(), Ok(self.sequence_state));
                self.sequence_stack.pop().unwrap();
            }
            ParserSequence::Repeat { expr, delim, min, max, last_pos } => {
                // If no progress was made, we've parsed as much as we're gonna get
                let first = if let Some(last_pos) = last_pos {
                    if *last_pos == self.sequence_state.pos {
                        assert_eq!(*min, 0, "Repeat rule made no progress");
                        self.sequence_stack.pop().unwrap();
                        return Ok(())
                    }
                    false
                } else {
                    true
                };

                // If we reached the maximum, parsed ok
                if let Some(max) = max {
                    if *max == 0 {
                        self.sequence_stack.pop().unwrap();
                        return Ok(())
                    }
                    *max -= 1;
                }

                *last_pos = Some(self.sequence_state.pos);
                let delim = *delim;
                let expr = *expr;

                // If we reached the minimum, parsing is optional
                if *min == 0 {
                    self.add_choice(ParserChoiceSub::RepeatOptional);
                } else {
                    *min -= 1;
                }

                // Add the next set of delim,expr to the stack. Skip delim if this is the first time.
                self.add_expr(expr);
                if !first {
                    self.add_expr(delim);
                }
            }
            ParserSequence::PositiveLookaheadEnd { sequence_state } => {
                self.sequence_state = *sequence_state;
                self.sequence_stack.pop().unwrap();
            },
            ParserSequence::NegativeLookaheadEnd { sequence_state } => {
                // If this sequence is encountered, parsing the negative lookahead expr succeeeded
                // We need to remove the negative lookahead choice from the choice stack then fail
                let e = E::new(self.sequence_state.pos.span_to(sequence_state.pos));
                self.sequence_stack.pop().unwrap();
                let ParserChoiceSub::NegativeLookaheadFail = self.drop_choice().choice else {
                    panic!()
                };
                self.fail(e)?;
            },
        }
        Ok(())
    }
}

pub enum ParserSequence<'arn, 'grm: 'arn> {
    Exprs(&'arn [RuleExpr<'arn, 'grm>]),
    Block(&'arn BlockState<'arn, 'grm>),
    PopChoice(#[cfg(debug_assertions)] usize),
    Repeat {
        expr: &'arn RuleExpr<'arn, 'grm>,
        delim: &'arn RuleExpr<'arn, 'grm>,
        min: u64,
        max: Option<u64>,
        last_pos: Option<Pos>,
    },
    CacheOk {
        key: CacheKey<'arn, 'grm>,
    },
    PositiveLookaheadEnd {
        sequence_state: SequenceState<'arn, 'grm>,
    },
    NegativeLookaheadEnd {
        sequence_state: SequenceState<'arn, 'grm>,
    },
}