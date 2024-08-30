mod primitives;

use crate::core::adaptive::{BlockState, Constructor, GrammarState, RuleId, RuleState};
use crate::core::cache::Allocs;
use crate::core::pos::Pos;
use crate::core::span::Span;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::{GrammarFile, RuleExpr};
use crate::META_GRAMMAR;
use std::marker::PhantomData;
use std::slice;

pub trait Action {}

pub struct ParserState<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> {
    allocs: Allocs<'arn>,
    input: &'grm str,

    sequence_stack: Vec<ParserSequence<'arn, 'grm>>,
    choice_stack: Vec<ParserChoice<'arn, 'grm>>,

    seq_state: SeqState<'arn, 'grm>,
    furthest_error: Option<(E, Pos)>,
}

#[derive(Copy, Clone)]
struct SeqState<'arn, 'grm: 'arn> {
    grammar_state: &'arn GrammarState<'arn, 'grm>,
    pos: Pos,
}

struct ParserSequence<'arn, 'grm: 'arn> {
    sequence: ParserSequenceSub<'arn, 'grm>,
}

enum ParserSequenceSub<'arn, 'grm: 'arn> {
    Exprs(&'arn [RuleExpr<'arn, 'grm>]),
}

struct ParserChoice<'arn, 'grm: 'arn> {
    choice: ParserChoiceSub<'arn, 'grm>,
    seq_state: SeqState<'arn, 'grm>,
}

enum ParserChoiceSub<'arn, 'grm: 'arn> {
    Blocks(
        &'arn [BlockState<'arn, 'grm>],
        &'arn [Constructor<'arn, 'grm>],
    ),
    Exprs(&'arn [RuleExpr<'arn, 'grm>]),
}

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    pub fn run_rule(
        rules: &'arn GrammarFile<'arn, 'grm>,
        rule: &str,
        allocs: Allocs<'arn>,
        input: &'grm str,
    ) -> Result<(), AggregatedParseError<'grm, E>> {
        let (grammar_state, meta_vars) = GrammarState::new_with_meta_grammar(allocs, rules);
        let grammar_state = allocs.alloc(grammar_state);

        let mut state = Self {
            allocs,
            input,

            choice_stack: vec![],
            sequence_stack: vec![],
            seq_state: SeqState {
                grammar_state,
                pos: Pos::start(),
            },
            furthest_error: None,
        };

        let start_rule = meta_vars
            .get(rule)
            .expect("Rule exists")
            .as_rule_id()
            .expect("Rule is a rule");
        state.run(start_rule)
    }

    //TODO &mut self this, needs to reset state at end of run
    pub fn run(mut self, start_rule: RuleId) -> Result<(), AggregatedParseError<'grm, E>> {
        self.parse_rule(start_rule);

        while let Some(s) = self.sequence_stack.last_mut() {
            match &mut s.sequence {
                ParserSequenceSub::Exprs(exprs) => {
                    //TODO use stdlib when slice::take_first stabilizes
                    let Some(expr) = take_first(exprs) else {
                        self.sequence_stack.pop();
                        continue;
                    };

                    match expr {
                        RuleExpr::RunVar(_, _) => todo!(),
                        RuleExpr::CharClass(cc) => {
                            let res = self.parse_char(|c| cc.contains(*c));
                            self.handle(res);
                        }
                        RuleExpr::Literal(lit) => todo!(),
                        RuleExpr::Repeat { .. } => todo!(),
                        RuleExpr::Sequence(seqs) => {
                            self.sequence_stack.push(ParserSequence {
                                sequence: ParserSequenceSub::Exprs(seqs),
                            });
                        }
                        RuleExpr::Choice(choices) => {
                            let (first_choice, rest_choices) =
                                choices.split_first().expect("Choices not empty");

                            self.choice_stack.push(ParserChoice {
                                choice: ParserChoiceSub::Exprs(rest_choices),
                                seq_state: self.seq_state,
                            });
                            self.sequence_stack.push(ParserSequence {
                                sequence: ParserSequenceSub::Exprs(slice::from_ref(first_choice)),
                            })
                        }
                        RuleExpr::NameBind(_, _) => todo!(),
                        RuleExpr::Action(_, _) => todo!(),
                        RuleExpr::SliceInput(_) => todo!(),
                        RuleExpr::PosLookahead(_) => todo!(),
                        RuleExpr::NegLookahead(_) => todo!(),
                        RuleExpr::This => todo!(),
                        RuleExpr::Next => todo!(),
                        RuleExpr::AtAdapt(_, _) => todo!(),
                        RuleExpr::Guid => todo!(),
                    }
                }
            }
        }

        // Sequence stack is empty, done parsing
        Ok(())
    }

    fn handle(&mut self, res: PResult) {
        match res {
            PResult::POk(span) => {
                todo!()
            }
            PResult::PErr => {
                //TODO fail
                todo!()
            }
        }
    }

    fn parse_rule(&mut self, rule: RuleId) {
        let rule_state: &'arn RuleState<'arn, 'grm> = self
            .seq_state
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
            seq_state: self.seq_state,
        });
        self.sequence_stack.push(ParserSequence {
            //TODO don't ignore attributes
            sequence: ParserSequenceSub::Exprs(slice::from_ref(&first_constructor.0 .1)),
        });
    }
}

fn take_first<'a, T>(slice: &mut &'a [T]) -> Option<&'a T> {
    let (first, rem) = slice.split_first()?;
    *slice = rem;
    Some(first)
}

pub enum PResult {
    POk(Span),
    PErr,
}
