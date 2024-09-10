mod add_rule;
mod cache;
mod debug;
mod fail;
mod parse_expr;
mod parse_sequence;
mod primitives;

use crate::core::adaptive::{BlockState, Constructor, GrammarState, RuleId};
use crate::core::cache::Allocs;
use crate::core::pos::Pos;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::{GrammarFile, RuleExpr};
use crate::parser::var_map::VarMap;
use crate::parser2::add_rule::BlockCtx;
use crate::parser2::cache::ParserCache;
use parse_sequence::ParserSequence;

pub trait Action {}

pub struct ParserState<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> {
    _allocs: Allocs<'arn>,
    input: &'grm str,
    cache: ParserCache<'arn, 'grm, E>,

    sequence_stack: Vec<ParserSequence<'arn, 'grm>>,
    choice_stack: Vec<ParserChoice<'arn, 'grm>>,

    sequence_state: SequenceState<'arn, 'grm>,
    furthest_error: Option<(E, Pos)>,
}

#[derive(Copy, Clone)]
pub struct SequenceState<'arn, 'grm: 'arn> {
    grammar_state: &'arn GrammarState<'arn, 'grm>,
    pos: Pos,
    vars: VarMap<'arn, 'grm>,
    block_ctx: Option<BlockCtx<'arn, 'grm>>,
}

pub struct ParserChoice<'arn, 'grm: 'arn> {
    choice: ParserChoiceSub<'arn, 'grm>,
    sequence_state: SequenceState<'arn, 'grm>,
    sequence_stack_len: usize,
}

pub enum ParserChoiceSub<'arn, 'grm: 'arn> {
    Blocks(&'arn [BlockState<'arn, 'grm>]),
    Constructors(&'arn [Constructor<'arn, 'grm>]),
    Exprs(&'arn [RuleExpr<'arn, 'grm>]),
    RepeatFail,
    NegativeLookaheadFail,
    LeftRecursionFail,
    LayoutFail,
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

        let state = Self {
            _allocs: allocs,
            input,
            cache: Default::default(),
            choice_stack: vec![],
            sequence_stack: vec![],
            sequence_state: SequenceState {
                grammar_state,
                pos: Pos::start(),
                vars,
                block_ctx: None,
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
        self.sequence_stack.push(ParserSequence::Layout { last_pos: None });
        self.add_rule(start_rule);

        while !self.sequence_stack.is_empty() {
            // self.print_debug_info();
            self.parse_sequence()?;
        }

        // Sequence stack is empty, done parsing
        // Check whether there is input left
        self.parse_eof()?;

        Ok(())
    }
}
