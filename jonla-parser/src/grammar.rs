use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GrammarFile<'grm> {
    #[serde(borrow)]
    pub rules: Vec<Rule<'grm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Rule<'grm> {
    pub name: &'grm str,
    pub blocks: Vec<Block<'grm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Block<'grm>(pub &'grm str, pub Vec<AnnotatedRuleExpr<'grm>>);

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct AnnotatedRuleExpr<'grm>(
    pub Vec<RuleAnnotation<'grm>>,
    #[serde(borrow)] pub RuleExpr<'grm>,
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
    Error(EscapedString<'grm>),
    DisableLayout,
    EnableLayout,
    DisableRecovery,
    EnableRecovery,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum RuleExpr<'grm> {
    Rule(&'grm str),
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
    Action(Box<Self>, RuleAction<'grm>),
    SliceInput(Box<Self>),
    PosLookahead(Box<Self>),
    NegLookahead(Box<Self>),
    AtGrammar,
    AtThis,
    AtNext,
    AtAdapt(RuleAction<'grm>, &'grm str),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum RuleAction<'grm> {
    Name(&'grm str),
    InputLiteral(EscapedString<'grm>),
    Construct(&'grm str, Vec<Self>),
    Cons(Box<Self>, Box<Self>),
    Nil(),
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct EscapedString<'grm>(Arc<Cow<'grm, str>>);

impl<'grm> EscapedString<'grm> {
    pub fn new(s: String) -> Self {
        Self(Arc::new(Cow::from(s)))
    }

    pub fn new_borrow(s: &'grm str) -> Self {
        Self(Arc::new(Cow::from(s)))
    }

    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.0.chars()
    }

    pub fn parse<F: FromStr>(&self) -> Result<F, F::Err> {
        self.0.parse()
    }
}

impl<'grm> Display for EscapedString<'grm> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
