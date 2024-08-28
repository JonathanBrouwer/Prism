use std::marker::PhantomData;
use crate::core::adaptive::{BlockState, Constructor, GrammarState, RuleId, RuleState};
use crate::core::cache::Allocs;
use crate::core::pos::Pos;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::{GrammarFile, RuleExpr};
use crate::META_GRAMMAR;

pub trait Action {

}

pub struct ParserState<'arn, 'grm: 'arn, E: ParseError<L= ErrorLabel<'grm>>> {
    allocs: Allocs<'arn>,
    input: &'grm str,

    sequence_stack: Vec<ParserSequence<'arn, 'grm>>,
    choice_stack: Vec<ParserChoice<'arn, 'grm>>,

    phantom_data: PhantomData<E>,
}

struct ParserSequence<'arn, 'grm: 'arn> {
    sequence: ParserSequenceSub<'arn, 'grm>,
}

enum ParserSequenceSub<'arn, 'grm: 'arn> {
    Expr(&'arn RuleExpr<'arn, 'grm>),
}

struct ParserChoice<'arn, 'grm: 'arn> {
    choice: ParserChoiceSub<'arn, 'grm>,
    grammar: &'arn GrammarState<'arn, 'grm>,
    pos: Pos,
}

enum ParserChoiceSub<'arn, 'grm: 'arn> {
    Blocks(&'arn [BlockState<'arn, 'grm>], &'arn [Constructor<'arn, 'grm>]),
}

impl<'arn, 'grm: 'arn, E: ParseError<L= ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    pub fn run_rule<A: Action>(
        rules: &'arn GrammarFile<'arn, 'grm>,
        rule: &str,
        allocs: Allocs<'arn>,
        input: &'grm str,
    ) -> Result<(), AggregatedParseError<'grm, E>> {
        let mut state = Self {
            allocs,
            input,
            phantom_data: PhantomData,

            choice_stack: vec![],
            sequence_stack: vec![],
        };
        state.run(rule, rules)
    }


    //TODO &mut self this, needs to reset state at end of run
    pub fn run(mut self, rule: &str, rules: &'arn GrammarFile<'arn, 'grm>,) -> Result<(), AggregatedParseError<'grm, E>> {
        let (grammar_state, meta_vars) = GrammarState::new_with_meta_grammar(self.allocs, rules);
        let grammar_state = self.allocs.alloc(grammar_state);
        
        let rule = meta_vars
            .get(rule)
            .expect("Rule exists")
            .as_rule_id()
            .expect("Rule is a rule");
        let start_pos = Pos::start();
        self.parse_rule(rule, grammar_state, start_pos);
        
        while let Some(s) = self.sequence_stack.pop() {
            
        }
        
        // Sequence stack is empty, done parsing
        Ok(())
    }
    
    
    fn parse_rule(
        &mut self,
        rule: RuleId,
        grammar: &'arn GrammarState<'arn, 'grm>,
        pos: Pos,
    ) {
        let rule_state: &'arn RuleState<'arn, 'grm> = grammar
            .get(rule)
            .unwrap_or_else(|| panic!("Rule not found: {rule}"));

        //TODO
        assert_eq!(rule_state.args.len(), 0);

        // Push remaining blocks
        let (first_block, rest_blocks) = rule_state.blocks.split_first().expect("Blocks not empty");
        let (first_constructor, rest_constructors) = first_block.constructors.split_first().expect("Constructors not empty");
        self.choice_stack.push(ParserChoice {
            choice: ParserChoiceSub::Blocks(&rest_blocks, rest_constructors),
            grammar,
            pos,
        });
        self.sequence_stack.push(ParserSequence {
            //TODO don't ignore attributes
            sequence: ParserSequenceSub::Expr(&first_constructor.0.1),
        });
    }
}

pub enum PResult<E: ParseError> {
    POk {
        start_span: Pos,
        err_span: Pos,
        furthest_error: Option<(E, Pos)>,
    },
    PErr {
        error: E,
        furthest_error: Pos,
    }
}
