mod adaptive;
mod lambda;
mod layout;
mod left_recursion;
mod list;
mod literal;
mod lookahead;
mod minor;
mod parser_tests;
mod repeat;
mod infinite;

macro_rules! parse_test {
    (name: $name:ident syntax: $syntax:literal passing tests: $($input_pass:literal => $expected:literal)* failing tests: $($input_fail:literal)*) => {
        #[allow(unused_imports)]
        #[test]
        fn $name() {
            use jonla_parser::grammar;
            use jonla_parser::grammar::GrammarFile;
            use jonla_parser::parser_core::error::empty_error::EmptyError;
            use jonla_parser::parser_core::parser::Parser;
            use jonla_parser::parser_core::presult::PResult;
            use jonla_parser::parser_core::presult::PResult::*;
            use jonla_parser::parser_core::stream::StringStream;
            use jonla_parser::parser_sugar::error_printer::*;
            use jonla_parser::parser_sugar::parser_rule::parser_rule;
            use jonla_parser::parser_sugar::parser_rule::ParserContext;
            use std::collections::HashMap;
            use jonla_parser::parser_sugar::run::run_parser_rule;
            use jonla_parser::parse_grammar;
            use jonla_parser::parser_core::error::set_error::SetError;

            let syntax: &'static str = $syntax;
            let grammar: GrammarFile = match parse_grammar::<SetError<_>>(syntax) {
                Ok(ok) => ok,
                Err(es) => {
                    for e in es {
                        print_set_error(e, "grammar", syntax, true);
                    }
                    panic!();
                }
            };

            $(
            let input: &'static str = $input_pass;
            println!("== Parsing (should be ok): {}", input);

            let stream: StringStream = StringStream::new(input);

            match run_parser_rule(&grammar, "start", stream) {
                Ok(o) => {
                    let got = o.1.to_string(input);
                    assert_eq!($expected, got);
                }
                Err(es) => {
                    for e in es {
                        // print_set_error(e, "tests", input, true);
                        print_tree_error(e, "tests", input, true);
                    }
                    panic!();
                }
            }
            )*

            $(
            let input: &'static str = $input_fail;
            println!("== Parsing (should be fail): {}", input);

            let stream: StringStream = StringStream::new(input);
            match run_parser_rule::<EmptyError<_>>(&grammar, "start", stream) {
                Ok(o) => {
                    let got = o.1.to_string(input);
                    println!("Got: {:?}", got);
                    panic!();
                }
                Err(_) => {}
            }
            )*
        }
    }
}

pub(crate) use parse_test;
