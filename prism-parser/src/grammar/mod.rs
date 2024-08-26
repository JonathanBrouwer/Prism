use crate::grammar::escaped_string::EscapedString;
use crate::grammar::rule_action::{RuleAction, RuleActionType};
use crate::grammar::serde_leak::*;
use serde::{Deserialize, Serialize};

pub mod action_result;
pub mod apply_action;
pub mod escaped_string;
pub mod from_action_result;
pub mod rule_action;
pub mod serde_leak;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GrammarFile<'arn, 'grm> {
    #[serde(borrow, with = "leak_slice")]
    pub rules: &'arn [Rule<'arn, 'grm>],
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Rule<'arn, 'grm> {
    pub name: &'grm str,
    pub adapt: bool,
    #[serde(with = "leak_slice")]
    pub args: &'arn [(&'grm str, RuleActionType<'arn, 'grm>)],
    pub rtrn_type: RuleActionType<'arn, 'grm>,
    #[serde(borrow, with = "leak_slice")]
    pub blocks: &'arn [Block<'arn, 'grm>],
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Block<'arn, 'grm> {
    pub name: &'grm str,
    pub adapt: bool,
    #[serde(borrow, with = "leak_slice")]
    pub constructors: &'arn [AnnotatedRuleExpr<'arn, 'grm>],
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AnnotatedRuleExpr<'arn, 'grm>(
    #[serde(borrow, with = "leak_slice")] pub &'arn [RuleAnnotation<'grm>],
    #[serde(borrow)] pub RuleExpr<'arn, 'grm>,
);

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct CharClass<'arn> {
    pub neg: bool,
    #[serde(borrow, with = "leak_slice")]
    pub ranges: &'arn [(char, char)],
}

impl CharClass<'_> {
    pub fn contains(&self, c: char) -> bool {
        self.ranges.iter().any(|range| range.0 <= c && c <= range.1) ^ self.neg
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum RuleAnnotation<'grm> {
    #[serde(borrow)]
    Error(EscapedString<'grm>),
    DisableLayout,
    EnableLayout,
    DisableRecovery,
    EnableRecovery,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RuleExpr<'arn, 'grm> {
    RunVar(
        &'grm str,
        #[serde(with = "leak_slice")] &'arn [RuleExpr<'arn, 'grm>],
    ),
    CharClass(CharClass<'arn>),
    Literal(EscapedString<'grm>),
    Repeat {
        #[serde(with = "leak")]
        expr: &'arn Self,
        min: u64,
        max: Option<u64>,
        #[serde(with = "leak")]
        delim: &'arn Self,
    },
    Sequence(#[serde(with = "leak_slice")] &'arn [RuleExpr<'arn, 'grm>]),
    Choice(#[serde(with = "leak_slice")] &'arn [RuleExpr<'arn, 'grm>]),
    NameBind(&'grm str, #[serde(with = "leak")] &'arn Self),
    Action(#[serde(with = "leak")] &'arn Self, RuleAction<'arn, 'grm>),
    SliceInput(#[serde(with = "leak")] &'arn Self),
    PosLookahead(#[serde(with = "leak")] &'arn Self),
    NegLookahead(#[serde(with = "leak")] &'arn Self),
    This,
    Next,
    AtAdapt(&'grm str, &'grm str),
    Guid,
}
