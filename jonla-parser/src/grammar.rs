use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct GrammarFile<'input> {
    pub rules: HashMap<&'input str, Rule<'input>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Rule<'input> {
    pub name: &'input str,
    pub blocks: Blocks<'input>,
}

pub type Blocks<'input> = Vec<Block<'input>>;
pub type Block<'input> = Constructors<'input>;
pub type Constructors<'input> = Vec<AnnotatedRuleExpr<'input>>;
pub type AnnotatedRuleExpr<'input> = (Vec<RuleAnnotation<'input>>, RuleExpr<'input>);

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct CharClass {
    pub ranges: Vec<(char, char)>,
}

impl CharClass {
    pub fn contains(&self, c: char) -> bool {
        self.ranges.iter().any(|range| range.0 <= c && c <= range.1)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAnnotation<'input> {
    Error(&'input str),
    NoLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleExpr<'input> {
    Rule(&'input str),
    CharClass(CharClass),
    Literal(&'input str),
    Repeat {
        expr: Box<RuleExpr<'input>>,
        min: u64,
        max: Option<u64>,
        delim: Box<RuleExpr<'input>>,
    },
    Sequence(Vec<RuleExpr<'input>>),
    Choice(Vec<RuleExpr<'input>>),
    NameBind(&'input str, Box<RuleExpr<'input>>),
    Action(Box<RuleExpr<'input>>, RuleAction<'input>),
    SliceInput(Box<RuleExpr<'input>>),
    PosLookahead(Box<RuleExpr<'input>>),
    NegLookahead(Box<RuleExpr<'input>>),
    AtThis,
    AtNext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction<'input> {
    Name(&'input str),
    InputLiteral(&'input str),
    Construct(&'input str, Vec<RuleAction<'input>>),
    Cons(Box<RuleAction<'input>>, Box<RuleAction<'input>>),
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

        pub rule toplevel() -> GrammarFile<'input> = rules:(__ r:prule() __ {r})* { GrammarFile{ rules: rules.into_iter().map(|rule| (rule.name, rule)).collect() } }

        //Rule
        rule prule() -> Rule<'input> =
            "rule" _ name:identifier() _ ":" _n() blocks:prule_blocks() { Rule{name, blocks } } /
            "rule" _ name:identifier() _ "=" _ expr:prule_expr() _n() { Rule{name, blocks: vec![vec![(vec![], expr)]] } }

        rule prule_blocks() -> Blocks<'input> = precedence! {
            bs:( prule_constructors() ) ** ("--" _n()) { bs }
        }

        rule prule_constructors() -> Constructors<'input> = precedence! {
            cs:( prule_annotated_expr() ) ** __ { cs }
        }

        rule prule_annotated_expr() -> AnnotatedRuleExpr<'input> = precedence! {
            ans:(a:prule_annotation() _n() {a})* expr:prule_expr() _n() { (ans, expr) }
        }

        rule prule_annotation() -> RuleAnnotation<'input> = precedence! {
            "@" _ "error" _ "(" _ "\"" err:$(str_char()*) "\"" _ ")" { RuleAnnotation::Error(err) }
            "@" _ "nolayout" { RuleAnnotation::NoLayout }
        }

        rule prule_expr() -> RuleExpr<'input> = precedence! {
            a:prule_action() _ "<-" _ r:(@) { RuleExpr::Action(Box::new(r), a) }
            --
            x:@ _ "/" _ y:(@) { RuleExpr::Choice(vec![x, y]) }
            --
            x:@ _ y:(@) { RuleExpr::Sequence(vec![x,y]) }
            --
            n:identifier() _ ":" _ e:(@) { RuleExpr::NameBind(n, Box::new(e)) }
            --
            r:(@) "*" { RuleExpr::Repeat{ expr: Box::new(r), min: 0, max: None, delim: Box::new(RuleExpr::Sequence(vec![])) } }
            r:(@) "+" { RuleExpr::Repeat{ expr: Box::new(r), min: 1, max: None, delim: Box::new(RuleExpr::Sequence(vec![])) } }
            r:(@) "?" { RuleExpr::Repeat{ expr: Box::new(r), min: 0, max: Some(1), delim: Box::new(RuleExpr::Sequence(vec![])) } }
            --
            "\"" n:$(str_char()*) "\"" { RuleExpr::Literal(n) }
            "[" c:charclass() "]" { RuleExpr::CharClass(c) }
            "str" _ "(" _ r:prule_expr() _ ")" { RuleExpr::SliceInput(Box::new(r)) }
            "pos" _ "(" _ r:prule_expr() _ ")" { RuleExpr::PosLookahead(Box::new(r)) }
            "neg" _ "(" _ r:prule_expr() _ ")" { RuleExpr::NegLookahead(Box::new(r)) }
            "(" _ r:prule_expr() _ ")" { r }
            "@this" { RuleExpr::AtThis }
            "@next" { RuleExpr::AtNext }
            name:identifier() { RuleExpr::Rule(name) }
        }

        rule prule_action() -> RuleAction<'input> = precedence! {
            h:(@) _ "::" _ t:@ { RuleAction::Cons(Box::new(h), Box::new(t)) }
            --
            "[]" { RuleAction::Nil() }
            n:identifier() _ "(" args:(prule_action()**(_ "," _)) ")" { RuleAction::Construct(n, args) }
            "\"" n:$(str_char()*) "\"" { RuleAction::InputLiteral(n) }
            n:identifier() { RuleAction::Name(n) }
        }

        rule charclass() -> CharClass = rs:(_ r:charclass_part() _ {r})++"|" { CharClass { ranges: rs } }

        rule charclass_part() -> (char, char) =
            "'" c1:str_char() "'" _ "-" _ "'" c2:str_char() "'"  { (c1, c2) } /
            "'" c:str_char() "'" { (c, c) }

        rule str_char() -> char =
            [^ '\'' | '"'|'\\'] /
            "\\n" { '\n' } /
            "\\r" { '\r' } /
            "\\\"" { '"' } /
            "\\\'" { '\'' }
    }
}
