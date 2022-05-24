#[derive(Debug)]
pub struct GrammarFile<'input> {
    pub asts: Vec<Ast<'input>>,
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

peg::parser! {
    pub grammar grammar_def() for str {
        rule _ = [' ']*
        rule __ = [' ' | '\n']*

        rule identifier() -> &'input str
            = x: quiet!{$([ 'a'..='z' | 'A'..='Z' | '_' ]['a'..='z' | 'A'..='Z' | '0'..='9' | '_' ]*)} / expected!("identifier")

        pub rule toplevel() -> GrammarFile<'input> = asts: (__ a:ast() __ {a})* { GrammarFile{ asts } }

        rule ast() -> Ast<'input> = "ast" _ name:identifier() _ "{" constructors:(__ c:ast_constructor() {c})* __ "}" { Ast { name, constructors } }
        rule ast_constructor() -> AstConstructor<'input> = name:identifier() _ "(" _ args:ast_constructor_arg()**"," _ ")" _ "\n" { AstConstructor{ name, args } }
        rule ast_constructor_arg() -> (&'input str, &'input str) = _ name:identifier() _ ":" _ typ:identifier() _ { (name, typ) }
    }
}