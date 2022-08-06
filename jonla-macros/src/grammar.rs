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

#[derive(Debug, Clone, Serialize)]
pub struct Rule<'input> {
    pub name: &'input str,
    pub rtrn: AstType<'input>,
    pub body: Vec<RuleBody<'input>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleBody<'input> {
    #[serde(borrow)]
    pub annotations: Vec<RuleAnnotation<'input>>,
    pub expr: RuleExpr<'input>
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
pub enum RuleAnnotation<'input> {
    Error(&'input str),
    NoLayout
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction<'input> {
    Name(&'input str),
    InputLiteral(&'input str),
    Construct(&'input str, Vec<RuleAction<'input>>),
}

peg::parser! {
    pub grammar grammar_def() for str {
        // Generic
        rule identifier() -> &'input str
            = !reserved() s:quiet!{$([ 'a'..='z' | 'A'..='Z' | '_' ]['a'..='z' | 'A'..='Z' | '0'..='9' | '_' ]*)} {s} / expected!("identifier")

        rule reserved() = "end" / "str" / "rule" / "ast"

        rule _ = [' ']*
        rule _w() = [' ']+
        rule __ = [' ' | '\n']*
        rule _n() = [' ']* ("\n" [' ']*)+

        pub rule toplevel() -> GrammarFile<'input> = asts:(__ a:ast() __ {a})* __ rules:(__ r:prule() __ {r})* { GrammarFile{ asts, rules } }

        //Ast
        rule ast() -> Ast<'input> = "ast" _ name:identifier() _ ":" _n() constructors:(c:ast_constructor() {c})* { Ast { name, constructors } }
        rule ast_constructor() -> AstConstructor<'input> = name:identifier() _ "(" _ args:ast_constructor_arg()**"," _ ")" _n() { AstConstructor{ name, args } }
        rule ast_constructor_arg() -> (&'input str, AstType<'input>) = _ name:identifier() _ ":" _ typ:ast_constructor_type() _ { (name, typ) }
        rule ast_constructor_type() -> AstType<'input> =
            "Input" { AstType::Input } /
            "[" _ t:ast_constructor_type() _ "]" { AstType::List(box t) } /
            r:identifier() { AstType::Ast(r) }

        //Rule
        rule prule() -> Rule<'input> =
            "rule" _ name:identifier() _ "->" _ rtrn:ast_constructor_type() _ ":" _n() body:prule_body() { Rule{name, rtrn, body } } /
            "rule" _ name:identifier() _ "->" _ rtrn:ast_constructor_type() _ "=" _ expr:prule_expr() _n() { Rule{name, rtrn, body: vec![RuleBody{annotations: vec![], expr}] } }

        rule prule_body() -> Vec<RuleBody<'input>> = cs:prule_body_constr()* { cs }

        rule prule_body_constr() -> RuleBody<'input> = annotations:prule_annotation()* expr:prule_expr() _n() { RuleBody{annotations, expr} }

        rule prule_annotation() -> RuleAnnotation<'input> = precedence! {
            "@" _ "error" _ "(" _ "\"" err:$(str_char()*) "\"" _ ")" _n() { RuleAnnotation::Error(err) }
            "@" _ "nolayout" _n() { RuleAnnotation::NoLayout }
        }

        rule prule_expr() -> RuleExpr<'input> = precedence! {
            a:prule_action() _ "<-" _ r:(@) { RuleExpr::Action(box r, a) }
            --
            x:(@) _ "/" _ y:@ { RuleExpr::Choice(vec![x, y]) }
            --
            x:(@) _ y:@ { RuleExpr::Sequence(vec![x,y]) }
            --
            n:identifier() _ ":" _ e:(@) { RuleExpr::NameBind(n, box e) }
            --
            r:(@) "*" { RuleExpr::Repeat{ expr: box r, min: 0, max: None, delim: box RuleExpr::Sequence(vec![]) } }
            r:(@) "+" { RuleExpr::Repeat{ expr: box r, min: 1, max: None, delim: box RuleExpr::Sequence(vec![]) } }
            r:(@) "?" { RuleExpr::Repeat{ expr: box r, min: 0, max: Some(1), delim: box RuleExpr::Sequence(vec![]) } }
            --
            name:identifier() { RuleExpr::Rule(name) }
            "\"" n:$(str_char()*) "\"" { RuleExpr::Literal(n) }
            "[" c:charclass() "]" { RuleExpr::CharClass(c) }
            "str" _ "(" _ r:prule_expr() _ ")" { RuleExpr::SliceInput(box r) }
            "(" _ r:prule_expr() _ ")" { r }
        }

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
