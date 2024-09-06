use crate::core::adaptive::{BlockState, Constructor, RuleId, RuleState};
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::RuleExpr;
use crate::parser2::parse_sequence::ParserSequence;
use crate::parser2::{ParserChoiceSub, ParserState};
use std::slice;

pub type BlockCtx<'arn, 'grm> = &'arn [BlockState<'arn, 'grm>];

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
        self.add_blocks(rule_state.blocks)
    }

    pub fn add_blocks(&mut self, blocks: &'arn [BlockState<'arn, 'grm>]) {
        if blocks.len() == 0 {
            return
        }
        self.sequence_stack.push(ParserSequence::Blocks(&blocks));
    }

    pub fn add_constructors(&mut self, constructors: &'arn [Constructor<'arn, 'grm>], blocks: BlockCtx<'arn, 'grm>) {
        self.sequence_stack.push(ParserSequence::RestoreBlockCtx(self.sequence_state.block_ctx));
        self.sequence_state.block_ctx = Some(blocks);

        let (first_constructor, rest_constructors) =
            constructors.split_first().expect("Block not empty");
        if !rest_constructors.is_empty() {
            self.add_choice(ParserChoiceSub::Constructors(rest_constructors));
        }
        self.add_constructor(first_constructor);
    }

    pub fn add_constructor(&mut self, c: &'arn Constructor<'arn, 'grm>) {
        self.add_expr(&c.0 .1)
    }

    pub fn add_expr(&mut self, expr: &'arn RuleExpr<'arn, 'grm>) {
        self.sequence_stack
            .push(ParserSequence::Exprs(slice::from_ref(expr)));
    }
}
