use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GrammarFile {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Rule {
    pub name: String,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Block(pub String, pub Vec<AnnotatedRuleExpr>);

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct AnnotatedRuleExpr(pub Vec<RuleAnnotation>, pub RuleExpr);

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
pub enum RuleAnnotation {
    Error(String),
    NoLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum RuleExpr {
    Rule(String),
    CharClass(CharClass),
    Literal(String),
    Repeat {
        expr: Box<RuleExpr>,
        min: u64,
        max: Option<u64>,
        delim: Box<RuleExpr>,
    },
    Sequence(Vec<RuleExpr>),
    Choice(Vec<RuleExpr>),
    NameBind(String, Box<RuleExpr>),
    Action(Box<RuleExpr>, RuleAction),
    SliceInput(Box<RuleExpr>),
    PosLookahead(Box<RuleExpr>),
    NegLookahead(Box<RuleExpr>),
    AtGrammar,
    AtThis,
    AtNext,
    AtAdapt(RuleAction, String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum RuleAction {
    Name(String),
    InputLiteral(String),
    Construct(String, Vec<RuleAction>),
    Cons(Box<RuleAction>, Box<RuleAction>),
    Nil(),
}
