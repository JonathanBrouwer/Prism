use serde::{Deserialize, Serialize};
use crate::grammar::rule::Rule;
use crate::grammar::serde_leak::*;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct GrammarFile<'arn, 'grm> {
    #[serde(borrow, with = "leak_slice")]
    pub rules: &'arn [Rule<'arn, 'grm>],
}