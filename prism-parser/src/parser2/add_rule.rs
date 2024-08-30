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
    pub fn add_rule(&mut self, rule: RuleId) {
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

    pub fn add_constructor(&mut self, c: &'arn Constructor<'arn, 'grm>) {
        self.add_expr(&c.0.1)
    }

    pub fn add_expr(&mut self, expr: &'arn RuleExpr<'arn, 'grm>) {
        self.sequence_stack.push(ParserSequence {
            sequence: ParserSequenceSub::Exprs(slice::from_ref(expr)),
        });
    }
}