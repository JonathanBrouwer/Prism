use crate::grammar::serde_leak::*;
use serde::{Deserialize, Serialize};
use rule_block::RuleBlock;

pub mod charclass;
pub mod escaped_string;
pub mod from_action_result;
pub mod rule_action;
pub mod rule_annotation;
pub mod rule_expr;
pub mod serde_leak;
pub mod annotated_rule_expr;
pub mod rule_block;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct GrammarFile<'arn, 'grm> {
    #[serde(borrow, with = "leak_slice")]
    pub rules: &'arn [Rule<'arn, 'grm>],
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Rule<'arn, 'grm> {
    pub name: &'grm str,
    pub adapt: bool,
    #[serde(with = "leak_slice")]
    pub args: &'arn [(&'grm str, &'grm str)],
    #[serde(borrow, with = "leak_slice")]
    pub blocks: &'arn [RuleBlock<'arn, 'grm>],
    pub return_type: &'grm str,
}

