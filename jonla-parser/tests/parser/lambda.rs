use crate::parser::parse_test;

parse_test! {
name: lambda
syntax: r#"
rule layout = [' ' | '\n']

rule identifier:
    @error("Identifier")
    @disable_layout
    @str([ 'a'-'z' | 'A'-'Z' | '_' ] ['a'-'z' | 'A'-'Z' | '0'-'9' | '_' ]*)

rule term:
    -- let
    Let(n, t, v, b) <- "let" n:identifier ":" t:@next "=" v:@next ";" b:@this
    -- fun
    FunConstruct(x, t, r) <- x:identifier ":" t:@this "." r:@this
    FunType(n, at, bt) <- n:identifier ":" at:@this "->" bt:@this
    FunType("_", at, bt) <- at:@next "->" bt:@this
    -- apply
    FunDestruct(f, a) <- f:@this " " a:@next
    -- base
    Type() <- "Type"
    Var(n) <- n:identifier
    t <- "(" t:term ")"

rule start = term

"#
passing tests:
    "let f : Type -> Type = x:Type. x; f" => "Let('f', FunType('_', Type(), Type()), FunConstruct('x', Type(), Var('x')), Var('f'))"

failing tests:
}
