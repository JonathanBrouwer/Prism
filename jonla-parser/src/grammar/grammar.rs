use std::fmt::Debug;
use std::hash::Hash;
use crate::grammar::escaped_string::EscapedString;
use serde::{Deserialize, Serialize};
use crate::core::adaptive::{RuleId};
use crate::core::context::RawEnv;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(bound(deserialize = "A: Action<'grm>, 'grm: 'de"))]
pub struct GrammarFile<'grm, A> {
    #[serde(borrow)]
    pub rules: Vec<Rule<'grm, A>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(bound(deserialize = "A: Action<'grm>, 'grm: 'de"))]
pub struct Rule<'grm, A> {
    pub name: &'grm str,
    pub args: Vec<&'grm str>,
    pub blocks: Vec<Block<'grm, A>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(bound(deserialize = "A: Action<'grm>, 'grm: 'de"))]
pub struct Block<'grm, A>(pub &'grm str, pub Vec<AnnotatedRuleExpr<'grm, A>>);

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(bound(deserialize = "A: Action<'grm>, 'grm: 'de"))]
pub struct AnnotatedRuleExpr<'grm, A>(
    pub Vec<RuleAnnotation<'grm>>,
    #[serde(borrow)] pub RuleExpr<'grm, A>,
);

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(bound(deserialize = "A: Action<'grm>, 'grm: 'de"))]
pub enum RuleExpr<'grm, A> {
    Rule(&'grm str, Vec<A>),
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
    Action(Box<Self>, A),
    SliceInput(Box<Self>),
    PosLookahead(Box<Self>),
    NegLookahead(Box<Self>),
    AtGrammar,
    AtThis,
    AtNext,
    AtAdapt(A, &'grm str),
}

pub trait Action<'grm>: Debug + Clone + Serialize + Deserialize<'grm> + Eq + PartialEq + Hash {
    fn from_rule(r: RuleId) -> Self;
    fn eval_to_rule<'b>(e: &RawEnv<'b, 'grm, Self>) -> Option<RuleId>;
}