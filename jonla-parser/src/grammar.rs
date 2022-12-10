use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct GrammarFile {
    pub rules: HashMap<String, Rule>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Rule {
    pub name: String,
    pub blocks: Blocks,
}

pub type Blocks = Vec<Block>;
pub type Block = Constructors;
pub type Constructors = Vec<AnnotatedRuleExpr>;
pub type AnnotatedRuleExpr = (Vec<RuleAnnotation>, RuleExpr);

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAnnotation {
    Error(String),
    NoLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    Name(String),
    InputLiteral(String),
    Construct(String, Vec<RuleAction>),
    Cons(Box<RuleAction>, Box<RuleAction>),
    Nil(),
}

peg::parser! {
    pub grammar grammar_def() for str {
        // Generic
        rule identifier() -> &'input str
            = !reserved() s:quiet!{$([ 'a'..='z' | 'A'..='Z' | '_' ]['a'..='z' | 'A'..='Z' | '0'..='9' | '_' ]*)} {s} / expected!("identifier")

        rule reserved() = "end" / "str" / "rule" / "ast" / "neg" / "pos"

        rule _ = [' ']*
        rule _w() = [' ']+
        rule __ = [' ' | '\n']*
        rule _n() = [' ']* ("\n" [' ']*)+

        pub rule toplevel() -> GrammarFile = rules:(__ r:prule() __ {r})* { GrammarFile{ rules: rules.into_iter().map(|rule| (rule.name.clone(), rule)).collect() } }

        //Rule
        rule prule() -> Rule =
            "rule" _ name:identifier() _ ":" _n() blocks:prule_blocks() { Rule{ name: name.to_string(), blocks } } /
            "rule" _ name:identifier() _ "=" _ expr:prule_expr() _n() { Rule{name: name.to_string(), blocks: vec![vec![(vec![], expr)]] } }

        rule prule_blocks() -> Blocks = precedence! {
            bs:( prule_constructors() ) ++ ("--" _n()) { bs }
        }

        rule prule_constructors() -> Constructors = precedence! {
            cs:( prule_annotated_expr() )+ { cs }
        }

        rule prule_annotated_expr() -> AnnotatedRuleExpr = precedence! {
            ans:(a:prule_annotation() _n() {a})* expr:prule_expr() _n() { (ans, expr) }
        }

        rule prule_annotation() -> RuleAnnotation = precedence! {
            "@" _ "error" _ "(" _ "\"" err:$(str_char()*) "\"" _ ")" { RuleAnnotation::Error(err.to_string()) }
            "@" _ "nolayout" { RuleAnnotation::NoLayout }
        }

        rule prule_expr() -> RuleExpr = precedence! {
            a:prule_action() _ "<-" _ r:(@) { RuleExpr::Action(Box::new(r), a) }
            --
            x:@ _ "/" _ y:(@) { RuleExpr::Choice(vec![x, y]) }
            --
            x:@ _ y:(@) { RuleExpr::Sequence(vec![x,y]) }
            --
            n:identifier() _ ":" _ e:(@) { RuleExpr::NameBind(n.to_string(), Box::new(e)) }
            --
            r:(@) "*" { RuleExpr::Repeat{ expr: Box::new(r), min: 0, max: None, delim: Box::new(RuleExpr::Sequence(vec![])) } }
            r:(@) "+" { RuleExpr::Repeat{ expr: Box::new(r), min: 1, max: None, delim: Box::new(RuleExpr::Sequence(vec![])) } }
            r:(@) "?" { RuleExpr::Repeat{ expr: Box::new(r), min: 0, max: Some(1), delim: Box::new(RuleExpr::Sequence(vec![])) } }
            --
            "\"" n:$(str_char()*) "\"" { RuleExpr::Literal(n.to_string()) }
            "[" c:charclass() "]" { RuleExpr::CharClass(c) }
            "str" _ "(" _ r:prule_expr() _ ")" { RuleExpr::SliceInput(Box::new(r)) }
            "pos" _ "(" _ r:prule_expr() _ ")" { RuleExpr::PosLookahead(Box::new(r)) }
            "neg" _ "(" _ r:prule_expr() _ ")" { RuleExpr::NegLookahead(Box::new(r)) }
            "(" _ r:prule_expr() _ ")" { r }
            "@this" { RuleExpr::AtThis }
            "@next" { RuleExpr::AtNext }
            name:identifier() { RuleExpr::Rule(name.to_string()) }
        }

        rule prule_action() -> RuleAction = precedence! {
            h:(@) _ "::" _ t:@ { RuleAction::Cons(Box::new(h), Box::new(t)) }
            --
            "[]" { RuleAction::Nil() }
            n:identifier() _ "(" args:(prule_action()**(_ "," _)) ")" { RuleAction::Construct(n.to_string(), args) }
            "\"" n:$(str_char()*) "\"" { RuleAction::InputLiteral(n.to_string()) }
            n:identifier() { RuleAction::Name(n.to_string()) }
            "(" _ a:prule_action() _ ")" { a }
        }

        rule charclass() -> CharClass = neg:"^"? _ rs:(_ r:charclass_part() _ {r})**"|" { CharClass { neg: neg.is_some(), ranges: rs } }

        rule charclass_part() -> (char, char) =
            "'" c1:str_char() "'" _ "-" _ "'" c2:str_char() "'"  { (c1, c2) } /
            "'" c:str_char() "'" { (c, c) }

        rule str_char() -> char =
            [^ '\'' | '"' | '\\'] /
            "\\n" { '\n' } /
            "\\r" { '\r' } /
            "\\\"" { '"' } /
            "\\\'" { '\'' }
    }
}
