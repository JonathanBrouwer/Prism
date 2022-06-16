#[derive(Debug)]
pub struct GrammarFile<'input> {
    pub asts: Vec<Ast<'input>>,
    pub rules: Vec<Rule<'input>>,
}

#[derive(Debug)]
pub struct Ast<'input> {
    pub name: &'input str,
    pub constructors: Vec<AstConstructor<'input>>,
}

#[derive(Debug)]
pub struct AstConstructor<'input> {
    pub name: &'input str,
    pub args: Vec<(&'input str, &'input str)>
}

#[derive(Debug)]
pub struct Rule<'input> {
    pub name: &'input str,
    pub rtrn: &'input str,
    pub body: RuleBody<'input>,

}

#[derive(Debug)]
pub enum TrailingDelim {
    No,
    Maybe,
    Yes
}

#[derive(Debug)]
pub struct CharClass {
    ranges: Vec<(char, char)>
}

impl CharClass {
    pub fn contains(&self, c: char) -> bool {
        self.ranges.iter().any(|range| range.0 >= c && range.1 <= c)
    }
}

#[derive(Debug)]
pub enum RuleBody<'input> {
    Rule(&'input str),
    CharClass(CharClass),
    Literal(&'input str),
    Repeat{expr: Box<RuleBody<'input>>, min: u64, max: Option<u64>, delim: Box<RuleBody<'input>>, trailing_delim: TrailingDelim },
    Sequence(Vec<RuleBody<'input>>),
    Choice(Vec<RuleBody<'input>>),
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
        rule ast_constructor_arg() -> (&'input str, &'input str) = _ name:identifier() _ ":" _ typ:identifier() _ { (name, typ) }

        rule prule() -> Rule<'input> =
            "rule" _ name:identifier() _ "->" _ rtrn:identifier() _ "{" __ body:prule_body() __ "}" { Rule{name, rtrn, body } } /
            "rule" _ name:identifier() _ "->" _ rtrn:identifier() _ "=" _ body:prule_body() { Rule{name, rtrn, body } }

        rule prule_body() -> RuleBody<'input> =
            rs:(r:prule_body_1() {r})**<2,> (_ "/" _) { RuleBody::Sequence(rs) } /
            r:prule_body_1() { r }
        rule prule_body_1() -> RuleBody<'input> =
            rs:(r:prule_body_2() {r})**<0,> (_) { RuleBody::Sequence(rs) }
        rule prule_body_2() -> RuleBody<'input> =
            r:prule_body_3() "*" { RuleBody::Repeat{ expr: box r, min: 0, max: None, delim: box RuleBody::Sequence(vec![]), trailing_delim: TrailingDelim::No } } /
            r:prule_body_3() { r }
        rule prule_body_3() -> RuleBody<'input> =
            name:identifier() { RuleBody::Rule(name) } /
            "\"" n:$(str_char()*) "\"" { RuleBody::Literal(n) } /
            "[" c:charclass() "]" { RuleBody::CharClass(c) } /
            "(" _ r:prule_body() _ ")" { r }

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