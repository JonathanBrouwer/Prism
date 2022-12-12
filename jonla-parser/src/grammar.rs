use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GrammarFile {
    pub rules: HashMap<String, Rule>,
}

/// This exists since we want a stable serialization, and HashMap cannot provide that
#[derive(Serialize, Deserialize)]
struct GrammarFileForSerialization {
    rules: Vec<Rule>,
}

impl Serialize for GrammarFile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        GrammarFileForSerialization {
            rules: self
                .rules
                .values()
                .cloned()
                .sorted_by(|r1, r2| r1.name.cmp(&r2.name))
                .collect(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GrammarFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let gr = GrammarFileForSerialization::deserialize(deserializer)?;
        Ok(GrammarFile {
            rules: gr.rules.into_iter().map(|r| (r.name.clone(), r)).collect(),
        })
    }
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
    AtThis,
    AtNext,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum RuleAction {
    Name(String),
    InputLiteral(String),
    Construct(String, Vec<RuleAction>),
    Cons(Box<RuleAction>, Box<RuleAction>),
    Nil(),
}
