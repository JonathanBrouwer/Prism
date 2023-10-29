use std::marker::{PhantomData};
use crate::grammar::escaped_string::EscapedString;
use serde::{Deserialize, Serialize};
use crate::rule_action::action_result::ActionResult;

pub mod escaped_string;
pub mod from_action_result;
pub mod grammar_ar;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GrammarFile<'b, 'grm, A: Action<'b, 'grm>> {
    #[serde(borrow)]
    pub rules: Vec<Rule<'b, 'grm, A>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Rule<'b, 'grm, A: Action<'b, 'grm>> {
    pub name: &'grm str,
    pub args: Vec<&'grm str>,
    #[serde(borrow)] pub blocks: Vec<Block<'b, 'grm, A>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Block<'b, 'grm, A: Action<'b, 'grm>>(pub &'grm str, #[serde(borrow)] pub Vec<AnnotatedRuleExpr<'b, 'grm, A>>);

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct AnnotatedRuleExpr<'b, 'grm, A: Action<'b, 'grm>>(
    pub Vec<RuleAnnotation<'grm>>,
    #[serde(borrow)] pub RuleExpr<'b, 'grm, A>,
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
pub enum RuleExpr<'b, 'grm, A: Action<'b, 'grm>> {
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
    Action(Box<Self>, A, PhantomData<&'b str>),
    SliceInput(Box<Self>),
    PosLookahead(Box<Self>),
    NegLookahead(Box<Self>),
    AtThis,
    AtNext,
    AtAdapt(A, &'grm str),
}

pub trait Action<'b, 'grm>: Sized + 'b {
    fn parse_action(r: &'b ActionResult<'grm>, src: &'grm str) -> Option<Self>;
}

impl<'b, 'grm> Action<'b, 'grm> for &'b ActionResult<'grm> {
    fn parse_action(r: &'b ActionResult<'grm>, _src: &'grm str) -> Option<Self> {
        Some(r)
    }
}