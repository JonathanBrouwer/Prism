use jonla_macros::grammar;
use jonla_macros::grammar::GrammarFile;
use jonla_macros::grammar::RuleBodyExpr;
use jonla_macros::parser::core::error::empty_error::EmptyError;
use jonla_macros::parser::core::parser::Parser;
use jonla_macros::parser::core::presult::PResult;
use jonla_macros::parser::core::presult::PResult::*;
use jonla_macros::parser::core::primitives::full_input;
use jonla_macros::parser::core::stream::StringStream;
use jonla_macros::parser::error_printer::*;
use jonla_macros::parser::parser_rule::parser_rule;
use jonla_macros::parser::parser_rule::ParserContext;
use jonla_macros::parser::parser_state::ParserState;
use std::collections::HashMap;

macro_rules! parse_test {
    (name: $name:ident syntax: $syntax:literal passing tests: $($input_pass:literal => $expected:literal)* failing tests: $($input_fail:literal)*) => {
        #[test]
        fn $name() {
            let syntax: &'static str = $syntax;
            let grammar: GrammarFile = match grammar::grammar_def::toplevel(&syntax) {
                Ok(ok) => ok,
                Err(err) => {
                    panic!("{}", err);
                }
            };
            let rules: HashMap<&'static str, RuleBodyExpr<'static>> =
                grammar.rules.iter().map(|r| (r.name, r.body.clone())).collect();

            $(
            let input: &'static str = $input_pass;
            println!("== Parsing (should be ok): {}", input);

            let mut state = ParserState::new();
            let stream: StringStream = input.into();
            let result: PResult<_, _, _> = full_input(&parser_rule::<StringStream<'_>, _>(&rules, "start", &ParserContext::new())).parse(stream, &mut state);

            match result {
                POk(o, _, _) => {
                    let got = o.1.to_string(input);
                    assert_eq!($expected, got);
                }
                PErr(e, _) => {
                    // print_set_error(e, "test", input, true);
                    print_tree_error(e, "test", input, true);
                    panic!();
                }
            }
            )*

            $(
            let input: &'static str = $input_fail;
            println!("== Parsing (should be fail): {}", input);

            let mut state = ParserState::new();
            let stream: StringStream = input.into();
            let result: PResult<_, _, _> = full_input(&parser_rule::<StringStream<'_>, EmptyError<_>>(&rules, "start", &ParserContext::new())).parse(stream, &mut state);

            assert!(!result.is_ok());
            )*
        }
    }
}

parse_test! {
name: literal
syntax: r#"
    rule start -> Input:
        "lol"
    
    "#
passing tests:
    "lol" => "'lol'"
failing tests:
    "lolz"
    "loll"
    "lol "
    ""
    "l"
    "lo"
    " lol"
    "lo\nn"
}

parse_test! {
name: literal_indirect
syntax: r#"
    rule start -> Input:
        lol
    
    rule lol -> Input:
        "lol"
    
    "#
passing tests:
    "lol" => "'lol'"
failing tests:
    "lolz"
    "loll"
    "lol "
    ""
    "l"
    "lo"
    " lol"
    "lo\nn"
}

parse_test! {
name: charclass
syntax: r#"
    rule start -> Input:
        str([ 'w'-'z' | '8' | 'p'-'q' ])
    
    "#
passing tests:
    "8" => "'8'"
    "w" => "'w'"
    "x" => "'x'"
    "y" => "'y'"
    "z" => "'z'"
    "p" => "'p'"
    "q" => "'q'"

failing tests:
    "a"
    "b"
    "v"
    "7"
    "9"
    "o"
    "r"
    " "
    "w8"
    "8w"
}

parse_test! {
name: repeat_star
syntax: r#"
    rule start -> Input:
        str([ 'w'-'z' | '8' | 'p'-'q' ]*)
    
    "#
passing tests:
    "8" => "'8'"
    "w" => "'w'"
    "x" => "'x'"
    "y" => "'y'"
    "z" => "'z'"
    "p" => "'p'"
    "q" => "'q'"
    "" => "''"
    "8w"  => "'8w'"
    "w8" => "'w8'"
    "wxyz8pqpq8wz" => "'wxyz8pqpq8wz'"

failing tests:
    "a"
    "b"
    "v"
    "7"
    "9"
    "o"
    "r"
    " "
    "wxya"
    "w8 "
}
//
parse_test! {
name: repeat_plus
syntax: r#"
    rule start -> Input:
        str([ 'w'-'z' | '8' | 'p'-'q' ]+)
    
    "#
passing tests:
    "8" => "'8'"
    "w" => "'w'"
    "x" => "'x'"
    "y" => "'y'"
    "z" => "'z'"
    "p" => "'p'"
    "q" => "'q'"
    "8w"  => "'8w'"
    "w8" => "'w8'"
    "wxyz8pqpq8wz" => "'wxyz8pqpq8wz'"

failing tests:
    "a"
    "b"
    "v"
    "7"
    "9"
    "o"
    "r"
    " "
    "wxya"
    "w8 "
    ""
}

parse_test! {
name: repeat_option
syntax: r#"
    rule start -> Input:
        str([ 'w'-'z' | '8' | 'p'-'q' ]?)
    
    "#
passing tests:
    "8" => "'8'"
    "w" => "'w'"
    "x" => "'x'"
    "y" => "'y'"
    "z" => "'z'"
    "p" => "'p'"
    "q" => "'q'"
    "" => "''"

failing tests:
    "a"
    "b"
    "v"
    "7"
    "9"
    "o"
    "r"
    " "
    "wxya"
    "w8 "
    "8w"
    "w8"
    "wxyz8pqpq8wz"
}

parse_test! {
name: sequence
syntax: r#"
    rule start -> Input:
        str("a" ['w'-'y'] "q")
    
    "#
passing tests:
    "awq" => "'awq'"
    "axq" => "'axq'"
    "ayq" => "'ayq'"

failing tests:
    "a"
    "aw"
    "ax"
    "ay"
    "aqq"
    "aaq"
    "bwq"
    ""
    "awqq"
}

parse_test! {
name: choice
syntax: r#"
    rule start -> Input:
        "a" / ['w'-'y'] / "q"
    
    "#
passing tests:
    "a" => "'a'"
    "w" => "'w'"
    "y" => "'y'"
    "q" => "'q'"

failing tests:
    "aw"
    ""
    "b"
    "z"
    "wy"
    "wq"
}

parse_test! {
name: action
syntax: r#"
    ast Test:
        TestC(left: Input, right: Input)
    

    rule start -> Input:
        TestC(c, d) <- "a" c:['w'-'y'] d:"q"
    
    "#
passing tests:
    "awq" => "TestC('w', 'q')"
    "axq" => "TestC('x', 'q')"
    "ayq" => "TestC('y', 'q')"

failing tests:
    "a"
    "aw"
    "ax"
    "ay"
    "aqq"
    "aaq"
    "bwq"
    ""
    "awqq"
}

parse_test! {
name: list_ast
syntax: r#"
    ast Test:
        Leaf()
        Nodes(nodes: [Input])
    

    rule start -> Test:
        Nodes(ns) <- "(" ns:start* ")"
        Leaf() <- "x"
    
    "#
passing tests:
    "x" => "Leaf()"
    "()" => "Nodes([])"
    "(x)" => "Nodes([Leaf()])"
    "(xx)" => "Nodes([Leaf(), Leaf()])"
    "((x))" => "Nodes([Nodes([Leaf()])])"

failing tests:
    "xx"
    "(x"
    "x)"
    "(y)"
    "(x))"
    "((x)"
    ""
}

parse_test! {
name: list_rule
syntax: r#"
    ast Test:
        Leaf()
        Nodes(nodes: [Input])
    

    rule start -> [Test]:
        other*
    

    rule other -> Test:
        Nodes(ns) <- "(" ns:other* ")"
        Leaf() <- "x"
    
    "#
passing tests:
    "x" => "[Leaf()]"
    "()" => "[Nodes([])]"
    "(x)" => "[Nodes([Leaf()])]"
    "(xx)" => "[Nodes([Leaf(), Leaf()])]"
    "((x))" => "[Nodes([Nodes([Leaf()])])]"
    "xx" => "[Leaf(), Leaf()]"
    "" => "[]"

failing tests:
    "(x"
    "x)"
    "(y)"
    "(x))"
    "((x)"
}

parse_test! {
name: arith
syntax: r#"
    ast Expr:
        Add(l: Expr, r: Expr)
        Sub(l: Expr, r: Expr)
        Mul(l: Expr, r: Expr)
        Div(l: Expr, r: Expr)
        Pow(l: Expr, r: Expr)
        Neg(e: Expr)
        Num(n: Input)
    

    rule _ -> Input = [' ']*

    rule num -> Input:
        str(['0'-'9']+)
    

    rule start -> Expr:
        e <- _ e:expr _
    

    rule expr -> Expr:
        Add(l, r) <- l:expr2 _ "+" _ r:expr
        Sub(l, r) <- l:expr2 _ "-" _ r:expr
        expr2
    

    rule expr2 -> Expr:
        Mul(l, r) <- l:expr3 _ "*" _ r:expr2
        Div(l, r) <- l:expr3 _ "/" _ r:expr2
        expr3
    

    rule expr3 -> Expr:
        Pow(l, r) <- l:expr3 _ "^" _ r:expr4
        expr4
    

    rule expr4 -> Expr:
        Neg(e) <- "-" _ e:expr4
        Num(e) <- e:num
    
    "#
passing tests:
    "123" => "Num('123')"
    "5 * 4 + 20 * 4 - 50" => "Add(Mul(Num('5'), Num('4')), Sub(Mul(Num('20'), Num('4')), Num('50')))"
    "5 * 4 - 20 * 4 + 50" => "Sub(Mul(Num('5'), Num('4')), Add(Mul(Num('20'), Num('4')), Num('50')))"
    "-5 * -4 - -20 * -4 + -50" => "Sub(Mul(Neg(Num('5')), Neg(Num('4'))), Add(Mul(Neg(Num('20')), Neg(Num('4'))), Neg(Num('50'))))"
    "1 + 2 * 3" => "Add(Num('1'), Mul(Num('2'), Num('3')))"
    "1 * 2 + 3" => "Add(Mul(Num('1'), Num('2')), Num('3'))"
    "1 - 2 / 3" => "Sub(Num('1'), Div(Num('2'), Num('3')))"
    "1 / 2 - 3" => "Sub(Div(Num('1'), Num('2')), Num('3'))"
    "-8" => "Neg(Num('8'))"

failing tests:
    ""
    "1+"
    "+1"
}

parse_test! {
name: arith_layout
syntax: r#"
    ast Expr:
        Add(l: Expr, r: Expr)
        Sub(l: Expr, r: Expr)
        Mul(l: Expr, r: Expr)
        Div(l: Expr, r: Expr)
        Pow(l: Expr, r: Expr)
        Neg(e: Expr)
        Num(n: Input)
    

    rule layout -> Input = " "

    rule num -> Input:
        @nolayout
        str(['0'-'9']+)
    

    rule start -> Expr:
        e <- e:expr
    

    rule expr -> Expr:
        Add(l, r) <- l:expr2 "+" r:expr
        Sub(l, r) <- l:expr2 "-" r:expr
        expr2
    

    rule expr2 -> Expr:
        Mul(l, r) <- l:expr3 "*" r:expr2
        Div(l, r) <- l:expr3 "/" r:expr2
        expr3
    

    rule expr3 -> Expr:
        Pow(l, r) <- l:expr3 "^" r:expr4
        expr4
    

    rule expr4 -> Expr:
        Neg(e) <- "-" e:expr4
        Num(e) <- e:num
    
    "#
passing tests:
    "123" => "Num('123')"
    "5 * 4 + 20 * 4 - 50" => "Add(Mul(Num('5'), Num('4')), Sub(Mul(Num('20'), Num('4')), Num('50')))"
    "5 * 4 - 20 * 4 + 50" => "Sub(Mul(Num('5'), Num('4')), Add(Mul(Num('20'), Num('4')), Num('50')))"
    "-5 * -4 - -20 * -4 + -50" => "Sub(Mul(Neg(Num('5')), Neg(Num('4'))), Add(Mul(Neg(Num('20')), Neg(Num('4'))), Neg(Num('50'))))"
    "1 + 2 * 3" => "Add(Num('1'), Mul(Num('2'), Num('3')))"
    "1 * 2 + 3" => "Add(Mul(Num('1'), Num('2')), Num('3'))"
    "1 - 2 / 3" => "Sub(Num('1'), Div(Num('2'), Num('3')))"
    "1 / 2 - 3" => "Sub(Div(Num('1'), Num('2')), Num('3'))"
    "-8" => "Neg(Num('8'))"

failing tests:
    ""
    "1+"
    "+1"
}

parse_test! {
name: num_layout
syntax: r#"
    ast Expr:
        Neg(e: Expr)
        Num(n: Input)
    

    rule layout -> Input = " "

    rule num -> Input:
        @nolayout
        str(['0'-'9']+)
    

    rule start -> Expr:
        Neg(e) <- "-" e:start
        Num(e) <- e:num
    
    "#
passing tests:
    "123" => "Num('123')"
    "- 8" => "Neg(Num('8'))"

failing tests:
}
