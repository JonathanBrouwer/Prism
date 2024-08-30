mod parse_expr;
mod primitives;
mod cache;

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
use crate::parser2::cache::ParserCache;
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

struct ParserSequence<'arn, 'grm: 'arn> {
    sequence: ParserSequenceSub<'arn, 'grm>,
}

enum ParserSequenceSub<'arn, 'grm: 'arn> {
    Exprs(&'arn [RuleExpr<'arn, 'grm>]),
}

struct ParserChoice<'arn, 'grm: 'arn> {
    choice: ParserChoiceSub<'arn, 'grm>,
    sequence_state: SeqState<'arn, 'grm>,
}

enum ParserChoiceSub<'arn, 'grm: 'arn> {
    Blocks(
        &'arn [BlockState<'arn, 'grm>],
        &'arn [Constructor<'arn, 'grm>],
    ),
    Exprs(&'arn [RuleExpr<'arn, 'grm>]),
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
            match &mut s.sequence {
                ParserSequenceSub::Exprs(exprs) => {
                    //TODO use stdlib when slice::take_first stabilizes
                    let Some(expr) = parser2::take_first(exprs) else {
                        self.sequence_stack.pop();
                        continue;
                    };

                    match self.parse_expr(expr) {
                        Ok(()) => {}
                        Err(e) => {
                            self.fail(e)?;
                        }
                    }
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

    fn fail(&mut self, e: E) -> Result<(), AggregatedParseError<'grm, E>> {
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

    fn add_error(&mut self, e: E) {
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

    fn completely_fail(&mut self) -> AggregatedParseError<'grm, E> {
        AggregatedParseError {
            input: self.input,
            errors: vec![self.furthest_error.take().expect("Cannot fail without error").0],
        }
    }

    fn add_rule(&mut self, rule: RuleId) {
        let rule_state: &'arn RuleState<'arn, 'grm> = self
            .sequence_state
            .grammar_state
            .get(rule)
            .unwrap_or_else(|| panic!("Rule not found: {rule}"));

        //TODO
        assert_eq!(rule_state.args.len(), 0);

        // Push remaining blocks
        let (first_block, rest_blocks) = rule_state.blocks.split_first().expect("Blocks not empty");
        let (first_constructor, rest_constructors) = first_block
            .constructors
            .split_first()
            .expect("Constructors not empty");
        self.choice_stack.push(ParserChoice {
            choice: ParserChoiceSub::Blocks(&rest_blocks, rest_constructors),
            sequence_state: self.sequence_state,
        });
        self.add_constructor(first_constructor)
    }

    fn add_constructor(&mut self, c: &'arn Constructor<'arn, 'grm>) {
        self.add_expr(&c.0.1)
    }

    fn add_expr(&mut self, expr: &'arn RuleExpr<'arn, 'grm>) {
        self.sequence_stack.push(ParserSequence {
            sequence: ParserSequenceSub::Exprs(slice::from_ref(expr)),
        });
    }
}

fn take_first<'a, T>(slice: &mut &'a [T]) -> Option<&'a T> {
    let (first, rem) = slice.split_first()?;
    *slice = rem;
    Some(first)
}