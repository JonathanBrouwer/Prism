use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct GrammarFile<'input> {
    pub asts: Vec<Ast<'input>>,
    pub rules: Vec<Rule<'input>>,
}

#[derive(Debug, Clone)]
pub struct Ast<'input> {
    pub name: &'input str,
    pub constructors: Vec<AstConstructor<'input>>,
}

#[derive(Debug, Clone)]
pub struct AstConstructor<'input> {
    pub name: &'input str,
    pub args: Vec<(&'input str, AstType<'input>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstType<'input> {
    Input,
    Ast(&'input str),
    List(Box<AstType<'input>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule<'input> {
    pub name: &'input str,
    pub rtrn: AstType<'input>,
    pub body: RuleBody<'input>,
}

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
pub enum RuleBody<'input> {
    Rule(&'input str),
    CharClass(CharClass),
    Literal(&'input str),
    Repeat {
        expr: Box<RuleBody<'input>>,
        min: u64,
        max: Option<u64>,
        delim: Box<RuleBody<'input>>,
    },
    Sequence(Vec<RuleBody<'input>>),
    Choice(Vec<RuleBody<'input>>),
    NameBind(&'input str, Box<RuleBody<'input>>),
    Action(Box<RuleBody<'input>>, RuleAction<'input>),
    SliceInput(Box<RuleBody<'input>>),
    Error(Box<RuleBody<'input>>, &'input str),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction<'input> {
    Name(&'input str),
    InputLiteral(&'input str),
    Construct(&'input str, Vec<RuleAction<'input>>),
}

peg::parser! {
    pub grammar grammar_def() for str {
        rule _ = [' ']*
        rule __ = [' ' | '\n']*

        rule identifier() -> &'input str
            = x: quiet!{$([ 'a'..='z' | 'A'..='Z' | '_' ]['a'..='z' | 'A'..='Z' | '0'..='9' | '_' ]*)} / expected!("identifier")

        pub rule toplevel() -> GrammarFile<'input> = asts:(__ a:ast() __ {a})* __ rules:(__ r:prule() __ {r})* { GrammarFile{ asts, rules } }

        rule ast() -> Ast<'input> = "ast" _ name:identifier() _ "{" constructors:(__ c:ast_constructor() {c})* __ "}" { Ast { name, constructors } }
        rule ast_constructor() -> AstConstructor<'input> = name:identifier() _ "(" _ args:ast_constructor_arg()**"," _ ")" _ "\n" { AstConstructor{ name, args } }
        rule ast_constructor_arg() -> (&'input str, AstType<'input>) = _ name:identifier() _ ":" _ typ:ast_constructor_type() _ { (name, typ) }
        rule ast_constructor_type() -> AstType<'input> =
            "Input" { AstType::Input } /
            "[" _ t:ast_constructor_type() _ "]" { AstType::List(box t) } /
            r:identifier() { AstType::Ast(r) }

        rule prule() -> Rule<'input> =
            "rule" _ name:identifier() _ "->" _ rtrn:ast_constructor_type() _ "{" __ body:prule_body() __ "}" { Rule{name, rtrn, body } } /
            "rule" _ name:identifier() _ "->" _ rtrn:ast_constructor_type() _ "=" _ body:prule_body() { Rule{name, rtrn, body } }

        rule prule_body() -> RuleBody<'input> =
            rs:(r:prule_body_1a())**<2,>(__ "/" __) { RuleBody::Choice(rs) } /
            r:prule_body_1a() { r }
        rule prule_body_1a() -> RuleBody<'input> =
            r:prule_body_1() _ "{" _ a:prule_action() _ "/" _ "\"" _ err:$(str_char()*) _ "\"" _ "}" { RuleBody::Error(box RuleBody::Action(box r, a), err) } /
            r:prule_body_1() _ "{" _ "/" _ "\"" _ err:$(str_char()*) _ "\"" _ "}" { RuleBody::Error(box r, err) } /
            r:prule_body_1() _ "{" _ a:prule_action() _ "}" { RuleBody::Action(box r, a) } /
            r:prule_body_1() { r }
        rule prule_body_1() -> RuleBody<'input> =
            rs:(r:prule_body_2a() {r})**<2,> (_) { RuleBody::Sequence(rs) } /
            r:prule_body_2a() {r} /
            { RuleBody::Sequence(vec![]) }
        rule prule_body_2a() -> RuleBody<'input> =
            n:identifier() _ ":" _ r:prule_body_2() { RuleBody::NameBind(n, box r) } /
            r:prule_body_2() { r }
        rule prule_body_2() -> RuleBody<'input> =
            r:prule_body_3() "*" { RuleBody::Repeat{ expr: box r, min: 0, max: None, delim: box RuleBody::Sequence(vec![]) } } /
            r:prule_body_3() "+" { RuleBody::Repeat{ expr: box r, min: 1, max: None, delim: box RuleBody::Sequence(vec![]) } } /
            r:prule_body_3() "?" { RuleBody::Repeat{ expr: box r, min: 0, max: Some(1), delim: box RuleBody::Sequence(vec![]) } } /
            r:prule_body_3() { r }
        rule prule_body_3() -> RuleBody<'input> =
            name:identifier() { RuleBody::Rule(name) } /
            "\"" n:$(str_char()*) "\"" { RuleBody::Literal(n) } /
            "[" c:charclass() "]" { RuleBody::CharClass(c) } /
            "$" _ "(" _ r:prule_body() _ ")" { RuleBody::SliceInput(box r) } /
            "(" _ r:prule_body() _ ")" { r }

        rule prule_action() -> RuleAction<'input> =
            n:identifier() _ "(" args:(prule_action()**(_ "," _)) ")" { RuleAction::Construct(n, args) } /
            "\"" n:$(str_char()*) "\"" { RuleAction::InputLiteral(n) } /
            n:identifier() { RuleAction::Name(n) }

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
