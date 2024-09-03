use crate::core::adaptive::BlockState;
use crate::core::pos::Pos;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::error_printer::ErrorLabel::Debug;
use crate::error::ParseError;
use crate::grammar::RuleExpr;
use crate::parser2::cache::{CacheKey, CacheState};
use crate::parser2::fail::take_first;
use crate::parser2::{ParserChoiceSub, ParserState, SequenceState};
use crate::parser2::add_rule::BlockCtx;

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    /// Parse the top-most sequence from the sequence stack
    /// Returns an error if an unrecoverable error occurred
    pub fn parse_sequence(&mut self) -> Result<(), AggregatedParseError<'grm, E>> {
        let s = self
            .sequence_stack
            .last_mut()
            .expect("Precondition of method");
        match s {
            ParserSequence::Exprs(exprs, &ref blocks) => {
                //TODO use stdlib when slice::take_first stabilizes
                let Some(expr) = take_first(exprs) else {
                    self.sequence_stack.pop().unwrap();
                    return Ok(());
                };

                match self.parse_expr(expr, blocks) {
                    Ok(()) => {}
                    Err(e) => {
                        let () = self.fail(e)?;
                    }
                }
            }
            ParserSequence::PopChoice(..) => {
                self.drop_choice();
            }
            ParserSequence::Block(&ref block, &ref blocks) => {
                let key = CacheKey::new(
                    self.sequence_state.pos,
                    &block,
                    self.sequence_state.grammar_state,
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
                    return Ok(());
                }

                // Add left rec to sequence stack
                self.sequence_stack.pop().unwrap();
                self.sequence_stack.push(ParserSequence::LeftRecurse { key: key.clone(), last_pos: None, last_cache_state: self.cache.cache_state_get(), blocks });
                self.add_block(block, blocks);

                // Add initial error seed to prevent left recursion
                let err_span = self.sequence_state.pos.span_to(self.sequence_state.pos);
                let mut err = E::new(
                    err_span
                );
                err.add_label_explicit(Debug(err_span, "LEFTREC"));
                self.cache.insert(
                    key,
                    Err(err),
                );
            }
            ParserSequence::LeftRecurse {  ref key, last_pos, last_cache_state, blocks: &ref blocks } => {
                // If the last_pos is none and the cache was not read, no left-recursion is needed, since this is the first iteration and the corresponding cache entry was not touched
                // If the last_pos is some and equal to the current pos, parsing stagnated so we can return the current result
                if (last_pos.is_none() && !self.cache.is_read(key)) || last_pos.is_some_and(|last_pos| last_pos == self.sequence_state.pos) {
                    self.cache.insert(key.clone(), Ok(self.sequence_state));
                    self.sequence_stack.pop().unwrap();
                    return Ok(())
                }
                
                // Reset cache to state before this iteration, and plant the seed
                self.cache.cache_state_revert(*last_cache_state);
                self.cache.insert(key.clone(), Ok(self.sequence_state));
                
                // We need to make progress in the next iteration, to keep track of this we store last_pos
                *last_pos = Some(self.sequence_state.pos);

                let block = key.block.0;
                self.add_block(block, blocks);
            }
            ParserSequence::Repeat {
                expr: &ref expr,
                delim: &ref delim,
                min,
                max,
                last_pos,
                blocks: &ref blocks
            } => {
                // If no progress was made, we've parsed as much as we're gonna get
                let first = if let Some(last_pos) = last_pos {
                    if *last_pos == self.sequence_state.pos {
                        assert_eq!(*min, 0, "Repeat rule made no progress");
                        self.sequence_stack.pop().unwrap();
                        return Ok(());
                    }
                    false
                } else {
                    true
                };

                // If we reached the maximum, parsed ok
                if let Some(max) = max {
                    if *max == 0 {
                        self.sequence_stack.pop().unwrap();
                        return Ok(());
                    }
                    *max -= 1;
                }

                *last_pos = Some(self.sequence_state.pos);

                // If we reached the minimum, parsing is optional
                if *min == 0 {
                    self.add_choice(ParserChoiceSub::RepeatOptional);
                } else {
                    *min -= 1;
                }

                // Add the next set of delim,expr to the stack. Skip delim if this is the first time.
                self.add_expr(expr, blocks);
                if !first {
                    self.add_expr(delim, blocks);
                }
            }
            ParserSequence::PositiveLookaheadEnd { sequence_state } => {
                self.sequence_state = *sequence_state;
                self.sequence_stack.pop().unwrap();
            }
            ParserSequence::NegativeLookaheadEnd { sequence_state } => {
                // If this sequence is encountered, parsing the negative lookahead expr succeeeded
                // We need to remove the negative lookahead choice from the choice stack then fail
                let e = E::new(self.sequence_state.pos.span_to(sequence_state.pos));
                self.sequence_stack.pop().unwrap();
                let ParserChoiceSub::NegativeLookaheadFail = self.drop_choice().choice else {
                    panic!()
                };
                self.fail(e)?;
            }
        }
        Ok(())
    }
}

pub enum ParserSequence<'arn, 'grm: 'arn> {
    Exprs(&'arn [RuleExpr<'arn, 'grm>], BlockCtx<'arn, 'grm>),
    /// Refers only to the first block, the rest are there for context
    Block(&'arn BlockState<'arn, 'grm>, BlockCtx<'arn, 'grm>),
    PopChoice(#[cfg(debug_assertions)] usize),
    Repeat {
        expr: &'arn RuleExpr<'arn, 'grm>,
        delim: &'arn RuleExpr<'arn, 'grm>,
        min: u64,
        max: Option<u64>,
        last_pos: Option<Pos>,
        blocks: BlockCtx<'arn, 'grm>,
    },
    LeftRecurse {
        key: CacheKey<'arn, 'grm>,
        last_pos: Option<Pos>,
        last_cache_state: CacheState,
        blocks: BlockCtx<'arn, 'grm>,
    },
    PositiveLookaheadEnd {
        sequence_state: SequenceState<'arn, 'grm>,
    },
    NegativeLookaheadEnd {
        sequence_state: SequenceState<'arn, 'grm>,
    },
}
