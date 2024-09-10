use crate::core::adaptive::BlockState;
use crate::core::pos::Pos;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::error_printer::ErrorLabel::Debug;
use crate::error::ParseError;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::{CharClass, RuleExpr};
use crate::parser2::add_rule::BlockCtx;
use crate::parser2::cache::{CacheKey, CacheState};
use crate::parser2::fail::take_first;
use crate::parser2::{PResult, ParserChoiceSub, ParserState, SequenceState};

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    /// Parse the top-most sequence from the sequence stack
    /// Returns an error if an unrecoverable error occurred
    pub fn parse_sequence(&mut self) -> Result<(), AggregatedParseError<'grm, E>> {
        let s = self
            .sequence_stack
            .last_mut()
            .expect("Precondition of method");
        match s {
            ParserSequence::Exprs(exprs) => {
                //TODO use stdlib when slice::take_first stabilizes
                let Some(expr) = take_first(exprs) else {
                    self.sequence_stack.pop().unwrap();
                    return Ok(());
                };
                self.parse_expr(expr);
            }
            ParserSequence::PopChoice(..) => {
                self.drop_choice();
            }
            ParserSequence::Blocks(&ref blocks) => {
                let block = &blocks[0];
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
                self.sequence_stack.push(ParserSequence::LeftRecurse {
                    key: key.clone(),
                    last_seq_state: None,
                    last_cache_state: self.cache.cache_state_get(),
                    blocks,
                    old_ctx: self.sequence_state.block_ctx,
                });
                if blocks.len() > 1 {
                    self.add_choice(ParserChoiceSub::Blocks(&blocks[1..]));
                }
                self.add_constructors(block.constructors, blocks);

                // Add initial error seed to prevent left recursion
                let err_span = self.sequence_state.pos.span_to(self.sequence_state.pos);
                let mut err = E::new(err_span);
                err.add_label_explicit(Debug(err_span, "LEFTREC"));
                self.cache.insert(key, Err(err));
            }
            ParserSequence::LeftRecurse {
                ref key,
                last_seq_state,
                last_cache_state,
                blocks: &ref blocks,
                old_ctx,
            } => {
                // If the last_pos is none and the cache was not read, no left-recursion is needed, since this is the first iteration and the corresponding cache entry was not touched
                // If the last_pos is some and equal to the current pos, parsing stagnated so we can return the current result
                if (last_seq_state.is_none() && !self.cache.is_read(key))
                    || last_seq_state
                        .is_some_and(|last_seq_state| last_seq_state.pos >= self.sequence_state.pos)
                {
                    if let Some(last_seq_state) = last_seq_state {
                        self.sequence_state = *last_seq_state;
                    }
                    self.cache.insert(key.clone(), Ok(self.sequence_state));
                    self.sequence_state.block_ctx = old_ctx.take();
                    self.sequence_stack.pop().unwrap();
                    return Ok(());
                }

                // Reset cache to state before this iteration, and plant the seed
                self.cache.cache_state_revert(*last_cache_state);
                self.cache.insert(key.clone(), Ok(self.sequence_state));

                // We need to make progress in the next iteration, to keep track of this we store last_pos and restore sequence state
                *last_seq_state = Some(self.sequence_state);
                self.sequence_state.pos = key.pos;
                self.sequence_state.grammar_state = *key.grammar;

                let block = key.block.0;
                self.add_choice(ParserChoiceSub::LeftRecursionFail);
                self.add_constructors(block.constructors, blocks);
            }
            ParserSequence::Repeat {
                expr: &ref expr,
                delim: &ref delim,
                min,
                max,
                last_pos,
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
                    self.add_choice(ParserChoiceSub::RepeatFail);
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
            ParserSequence::Layout { last_pos } => {
                let Some(rule) = self.sequence_state.vars.get("layout") else {
                    self.sequence_stack.pop().unwrap();
                    return Ok(());
                };
                let Some(layout) = rule.as_rule_id() else {
                    panic!(
                        "Tried to run layout as a rule, but it did not refer to a rule."
                    );
                };
                
                if let Some(last_pos) = last_pos {
                    if *last_pos == self.sequence_state.pos {
                        self.sequence_stack.pop().unwrap();
                        return Ok(());
                    }
                }
                *last_pos = Some(self.sequence_state.pos);
                self.add_choice(ParserChoiceSub::LayoutFail);
                self.add_rule(layout);
            }
            ParserSequence::CharClass(&ref cc) => match self.parse_char(|c| cc.contains(*c)) {
                Ok(()) => {
                    self.sequence_stack.pop().unwrap();
                }
                Err(e) => self.fail(e)?,
            },
            ParserSequence::Literal(&ref lit) => match self.parse_chars(lit.chars()) {
                Ok(()) => {
                    self.sequence_stack.pop().unwrap();
                }
                Err(e) => {
                    self.fail(e)?
                }
            }
        }
        Ok(())
    }
}

pub enum ParserSequence<'arn, 'grm: 'arn> {
    Exprs(&'arn [RuleExpr<'arn, 'grm>]),
    Blocks(&'arn [BlockState<'arn, 'grm>]),
    PopChoice(#[cfg(debug_assertions)] usize),
    Repeat {
        expr: &'arn RuleExpr<'arn, 'grm>,
        delim: &'arn RuleExpr<'arn, 'grm>,
        min: u64,
        max: Option<u64>,
        last_pos: Option<Pos>,
    },
    LeftRecurse {
        key: CacheKey<'arn, 'grm>,
        last_seq_state: Option<SequenceState<'arn, 'grm>>,
        last_cache_state: CacheState,
        blocks: BlockCtx<'arn, 'grm>,
        old_ctx: Option<BlockCtx<'arn, 'grm>>,
    },
    PositiveLookaheadEnd {
        sequence_state: SequenceState<'arn, 'grm>,
    },
    NegativeLookaheadEnd {
        sequence_state: SequenceState<'arn, 'grm>,
    },
    Layout {
        last_pos: Option<Pos>,
    },
    CharClass(&'arn CharClass<'arn>),
    Literal(&'arn EscapedString<'grm>),
}
