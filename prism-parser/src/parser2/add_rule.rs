use crate::core::adaptive::{Constructor, RuleId, RuleState};
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::RuleExpr;
use crate::parser2::parse_sequence::ParserSequence;
use crate::parser2::{ParserChoiceSub, ParserState};
use std::slice;

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
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
        if !rest_blocks.is_empty() {
            self.add_choice(ParserChoiceSub::Blocks(&rest_blocks));
        }
        self.sequence_stack.push(ParserSequence::Block(first_block));
    }

    pub fn add_constructor(&mut self, c: &'arn Constructor<'arn, 'grm>) {
        self.add_expr(&c.0 .1)
    }

    pub fn add_expr(&mut self, expr: &'arn RuleExpr<'arn, 'grm>) {
        self.sequence_stack
            .push(ParserSequence::Exprs(slice::from_ref(expr)));
    }
}
