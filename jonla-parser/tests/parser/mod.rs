mod lambda;
mod layout;
mod left_recursion;
mod list;
mod literal;
mod lookahead;
mod minor;
mod parser_tests;
mod repeat;

macro_rules! parse_test {
    (name: $name:ident syntax: $syntax:literal passing tests: $($input_pass:literal => $expected:literal)* failing tests: $($input_fail:literal)*) => {
        #[allow(unused_imports)]
        #[test]
        fn $name() {
            use jonla_parser::grammar;
            use jonla_parser::grammar::GrammarFile;
            use jonla_parser::parser::core::error::empty_error::EmptyError;
            use jonla_parser::parser::core::parser::Parser;
            use jonla_parser::parser::core::presult::PResult;
            use jonla_parser::parser::core::presult::PResult::*;
            use jonla_parser::parser::core::stream::StringStream;
            use jonla_parser::parser::actual::error_printer::*;
            use jonla_parser::parser::actual::parser_rule::parser_rule;
            use jonla_parser::parser::actual::parser_rule::ParserContext;
            use std::collections::HashMap;
            use jonla_parser::parser::actual::parser_rule::run_parser_rule;
            use jonla_parser::parse_grammar;
            use jonla_parser::parser::core::error::set_error::SetError;

            let syntax: &'static str = $syntax;
            let grammar: GrammarFile = match parse_grammar::<SetError<_>>(syntax) {
                Ok(ok) => ok,
                Err(err) => {
                    print_set_error(err, "grammar", syntax, true);
                    panic!();
                }
            };

            $(
            let input: &'static str = $input_pass;
            println!("== Parsing (should be ok): {}", input);

            let stream: StringStream = input.into();

            match run_parser_rule(&grammar, "start", stream) {
                Ok(o) => {
                    let got = o.1.to_string(input);
                    assert_eq!($expected, got);
                }
                Err(e) => {
                    // print_set_error(e, "tests", input, true);
                    print_tree_error(e, "tests", input, true);
                    panic!();
                }
            }
            )*

            $(
            let input: &'static str = $input_fail;
            println!("== Parsing (should be fail): {}", input);

            let stream: StringStream = input.into();
            match run_parser_rule::<StringStream<'_>, EmptyError<_>>(&grammar, "start", stream) {
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
