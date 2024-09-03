use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::parser2::parse_sequence::ParserSequence;
use crate::parser2::{ParserChoice, ParserChoiceSub, ParserState};
use std::cmp::Ordering;

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    pub fn fail(&mut self, e: E) -> Result<(), AggregatedParseError<'grm, E>> {
        self.add_error(e);

        while let Some(s) = self.choice_stack.last_mut() {
            self.sequence_state = s.sequence_state;
            self.sequence_stack.truncate(s.sequence_stack_len);
            match &mut s.choice {
                ParserChoiceSub::Blocks(bs) => {
                    if bs.is_empty() {
                        self.drop_choice();
                        continue;
                    }
                    self.sequence_stack.push(ParserSequence::Block(bs));
                    *bs = &bs[1..];
                }
                ParserChoiceSub::Constructors(cs, &ref bs) => {
                    let Some(c) = take_first(cs) else {
                        self.drop_choice();
                        continue;
                    };
                    self.add_constructor(c, bs);
                }
                ParserChoiceSub::Exprs(exprs, &ref bs) => {
                    let Some(expr) = take_first(exprs) else {
                        self.drop_choice();
                        continue;
                    };
                    self.add_expr(expr, bs);
                }
                ParserChoiceSub::RepeatOptional => {
                    self.drop_choice();
                }
                ParserChoiceSub::NegativeLookaheadFail => {
                    self.drop_choice();
                }
            }
            return Ok(());
        }

        Err(self.completely_fail())
    }

    pub fn add_error(&mut self, e: E) {
        match &mut self.furthest_error {
            None => self.furthest_error = Some((e, self.sequence_state.pos)),
            Some((cur_err, cur_pos)) => match self.sequence_state.pos.cmp(cur_pos) {
                Ordering::Less => {}
                Ordering::Equal => *cur_err = cur_err.clone().merge(e),
                Ordering::Greater => {
                    *cur_pos = self.sequence_state.pos;
                    *cur_err = e;
                }
            },
        }
    }

    pub fn completely_fail(&mut self) -> AggregatedParseError<'grm, E> {
        AggregatedParseError {
            input: self.input,
            errors: vec![
                self.furthest_error
                    .take()
                    .expect("Cannot fail without error")
                    .0,
            ],
        }
    }

    pub fn add_choice(&mut self, choice: ParserChoiceSub<'arn, 'grm>) {
        self.sequence_stack.push(ParserSequence::PopChoice(
            #[cfg(debug_assertions)]
            self.choice_stack.len(),
        ));
        self.choice_stack.push(ParserChoice {
            choice,
            sequence_state: self.sequence_state,
            sequence_stack_len: self.sequence_stack.len(),
        });
    }

    pub fn drop_choice(&mut self) -> ParserChoice<'arn, 'grm> {
        let choice = self.choice_stack.pop().unwrap();
        let seq = self.sequence_stack.pop().unwrap();
        #[cfg(debug_assertions)]
        {
            let ParserSequence::PopChoice(expected_stack_size) = seq else {
                panic!()
            };
            assert_eq!(expected_stack_size, self.choice_stack.len());
        }
        choice
    }
}

pub fn take_first<'a, T>(slice: &mut &'a [T]) -> Option<&'a T> {
    let (first, rem) = slice.split_first()?;
    *slice = rem;
    Some(first)
}
