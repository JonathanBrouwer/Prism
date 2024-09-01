mod parse_expr;
mod primitives;
mod cache;
mod fail;
mod add_rule;

use std::cmp::Ordering;
use crate::core::adaptive::{BlockState, Constructor, GrammarState, RuleId, RuleState};
use crate::core::cache::Allocs;
use crate::core::pos::Pos;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::{GrammarFile, RuleExpr};
use crate::parser2;
use std::slice;
use crate::core::span::Span;
use crate::parser2::cache::{CacheKey, ParserCache};
use crate::parser2::fail::{take_first};
use crate::parser::var_map::VarMap;

pub trait Action {}

pub struct ParserState<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> {
    allocs: Allocs<'arn>,
    input: &'grm str,
    cache: ParserCache<'arn, 'grm, E>,

    sequence_stack: Vec<ParserSequence<'arn, 'grm>>,
    choice_stack: Vec<ParserChoice<'arn, 'grm>>,

    sequence_state: SeqState<'arn, 'grm>,
    furthest_error: Option<(E, Pos)>,
}

#[derive(Copy, Clone)]
struct SeqState<'arn, 'grm: 'arn> {
    grammar_state: &'arn GrammarState<'arn, 'grm>,
    pos: Pos,
    vars: VarMap<'arn, 'grm>
}

enum ParserSequence<'arn, 'grm: 'arn> {
    Exprs(&'arn [RuleExpr<'arn, 'grm>]),
    Block(&'arn BlockState<'arn, 'grm>),
    PopChoice,
    Repeat {
        expr: &'arn RuleExpr<'arn, 'grm>,
        delim: &'arn RuleExpr<'arn, 'grm>,
        min: u64,
        max: Option<u64>,
        last_pos: Option<Pos>,
    },
}

struct ParserChoice<'arn, 'grm: 'arn> {
    choice: ParserChoiceSub<'arn, 'grm>,
    sequence_state: SeqState<'arn, 'grm>,
    sequence_stack_len: usize,
}

enum ParserChoiceSub<'arn, 'grm: 'arn> {
    Blocks(&'arn [BlockState<'arn, 'grm>]),
    Constructors(&'arn [Constructor<'arn, 'grm>]),
    Exprs(&'arn [RuleExpr<'arn, 'grm>]),
    RepeatOptional,
}

pub type PResult<E> = Result<(), E>;

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    pub fn run_rule(
        rules: &'arn GrammarFile<'arn, 'grm>,
        rule: &str,
        allocs: Allocs<'arn>,
        input: &'grm str,
    ) -> Result<(), AggregatedParseError<'grm, E>> {
        let (grammar_state, vars) = GrammarState::new_with_meta_grammar(allocs, rules);
        let grammar_state = allocs.alloc(grammar_state);

        let mut state = Self {
            allocs,
            input,
            cache: Default::default(),
            choice_stack: vec![],
            sequence_stack: vec![],
            sequence_state: SeqState {
                grammar_state,
                pos: Pos::start(),
                vars,
            },
            furthest_error: None,
        };

        let start_rule = vars
            .get(rule)
            .expect("Rule exists")
            .as_rule_id()
            .expect("Rule is a rule");
        state.run(start_rule)
    }

    //TODO &mut self this, needs to reset state at end of run
    pub fn run(mut self, start_rule: RuleId) -> Result<(), AggregatedParseError<'grm, E>> {
        self.add_rule(start_rule);

        while let Some(s) = self.sequence_stack.last_mut() {
            todo!("Choice stack needs to be truncated");
            match s {
                ParserSequence::Exprs(exprs) => {
                    //TODO use stdlib when slice::take_first stabilizes
                    let Some(expr) = take_first(exprs) else {
                        self.sequence_stack.pop();
                        continue;
                    };

                    match self.parse_expr(expr) {
                        Ok(()) => {}
                        Err(e) => {
                            let () = self.fail(e)?;
                        }
                    }
                }
                ParserSequence::PopChoice => {
                    self.choice_stack.pop();
                    self.sequence_stack.pop();
                }
                ParserSequence::Block(block) => {
                    let key = CacheKey::new(
                        self.sequence_state.pos,
                        block,
                        self.sequence_state.grammar_state
                    );
                    if let Some(cached_result) = self.cache.get(&key) {
                        match cached_result {
                            Ok(()) => {}
                            Err(e) => {
                                let e = e.clone();
                                let () = self.fail(e)?;
                            }
                        }
                        return Ok(())
                    }
                    self.cache.insert(key, PResult::Err(E::new(self.sequence_state.pos.span_to(self.sequence_state.pos))));
                    let (first_constructor, rest_constructors) = block.constructors.split_first().expect("Block not empty");
                    self.sequence_stack.pop();
                    self.add_choice(ParserChoiceSub::Constructors(rest_constructors));
                    self.add_constructor(first_constructor);
                }
                ParserSequence::Repeat { expr, delim, min, max, last_pos } => {
                    // If no progress was made, we've parsed as much as we're gonna get
                    let first = if let Some(last_pos) = last_pos {
                        if *last_pos == self.sequence_state.pos {
                            assert_eq!(*min, 0, "Repeat rule made no progress");
                            self.sequence_stack.pop();
                            return Ok(())
                        }
                        false
                    } else {
                        true
                    };

                    // If we reached the maximum, return ok
                    if let Some(max) = max {
                        if *max == 0 {
                            self.sequence_stack.pop();
                            return Ok(())
                        }
                        *max -= 1;
                    }

                    *last_pos = Some(self.sequence_state.pos);
                    let delim = *delim;
                    let expr= *expr;

                    // If we reached the minimum, parsing is optional
                    if *min == 0 {
                        self.add_choice(ParserChoiceSub::RepeatOptional);
                    } else {
                        *min -= 1;
                    }

                    // Add the next set of delim,expr to the stack. Skip delim if this is the first time.
                    if !first {
                        self.add_expr(delim);
                    }
                    self.add_expr(expr);
                }
            }
        }

        // Sequence stack is empty, done parsing
        // Check whether there is input left
        if self.sequence_state.pos.next(self.input).1.is_some() {
            self.add_error(E::new(self.sequence_state.pos.span_to(Pos::end(self.input))));
            return Err(self.completely_fail());
        }

        Ok(())
    }

    pub fn add_choice(&mut self, choice: ParserChoiceSub<'arn, 'grm>) {
        self.sequence_stack.push(ParserSequence::PopChoice);
        self.choice_stack.push(ParserChoice {
            choice,
            sequence_state: self.sequence_state,
            sequence_stack_len: self.sequence_stack.len(),
        })
    }
}

