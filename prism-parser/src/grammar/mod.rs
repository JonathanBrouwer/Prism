use crate::grammar::serde_leak::leak;
use crate::grammar::escaped_string::EscapedString;
use serde::{Deserialize, Serialize};
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::state::PState;
use crate::error::ParseError;
use crate::rule_action::RuleAction;

pub mod escaped_string;
pub mod from_action_result;
pub mod serde_leak;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarFile<'arn, 'grm> {
    #[serde(borrow)]
    pub rules: Vec<Rule<'arn, 'grm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule<'arn, 'grm> {
    pub name: &'grm str,
    pub args: Vec<&'grm str>,
    #[serde(borrow)]
    pub blocks: Vec<Block<'arn, 'grm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block<'arn, 'grm>(
    pub &'grm str,
    #[serde(borrow)] pub Vec<AnnotatedRuleExpr<'arn, 'grm>>,
);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedRuleExpr<'arn, 'grm>(
    pub Vec<RuleAnnotation<'grm>>,
    #[serde(borrow)] pub RuleExpr<'arn, 'grm>,
);

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct CharClass {
    pub neg: bool,
    pub ranges: Vec<(char, char)>,
}

impl CharClass {
    pub fn contains(&self, c: char) -> bool {
        self.ranges.iter().any(|range| range.0 <= c && c <= range.1) ^ self.neg
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum RuleAnnotation<'grm> {
    #[serde(borrow)]
    Error(EscapedString<'grm>),
    DisableLayout,
    EnableLayout,
    DisableRecovery,
    EnableRecovery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleExpr<'arn, 'grm> {
    RunVar(&'grm str, Vec<Self>),
    CharClass(CharClass),
    Literal(EscapedString<'grm>),
    Repeat {
        expr: Box<Self>,
        min: u64,
        max: Option<u64>,
        delim: Box<Self>,
    },
    Sequence(Vec<Self>),
    Choice(Vec<Self>),
    NameBind(&'grm str, Box<Self>),
    Action(Box<Self>, RuleAction<'arn, 'grm>),
    SliceInput(Box<Self>),
    PosLookahead(Box<Self>),
    NegLookahead(Box<Self>),
    This,
    Next,
    AtAdapt(RuleAction<'arn, 'grm>, &'grm str),
    Guid,
    // Test(#[serde(with= "leak")] &'grm RuleExpr<'arn, 'grm>)
}
