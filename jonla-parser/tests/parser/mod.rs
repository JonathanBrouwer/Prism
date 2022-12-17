mod adaptive;
mod infinite;
mod lambda;
mod layout;
mod left_recursion;
mod list;
mod literal;
mod lookahead;
mod minor;
mod parser_tests;
mod recovery;
mod repeat;

macro_rules! parse_test {
    (name: $name:ident syntax: $syntax:literal passing tests: $($input_pass:literal => $expected:literal)* failing tests: $($input_fail:literal $(=> $errors:literal)?)*) => {
        #[allow(unused_imports)]
        #[allow(unused_variables)]
        #[test]
        fn $name() {
            use jonla_parser::parse_grammar;
            use jonla_parser::grammar::grammar::GrammarFile;
            use jonla_parser::grammar;
            use jonla_parser::error::empty_error::EmptyError;
            use jonla_parser::core::parser::Parser;
            use jonla_parser::core::presult::PResult;
            use jonla_parser::core::presult::PResult::*;
            use jonla_parser::core::stream::StringStream;
            use jonla_parser::error::error_printer::*;
            use jonla_parser::grammar::parser_rule::parser_rule;
            use jonla_parser::core::context::ParserContext;
            use std::collections::HashMap;
            use jonla_parser::grammar::run::run_parser_rule;
            use jonla_parser::error::set_error::SetError;
            use crate::parser::errors_to_str;

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
            match run_parser_rule::<SetError<_>>(&grammar, "start", stream) {
                Ok(o) => {
                    let got = o.1.to_string(input);
                    println!("Got: {:?}", got);
                    panic!();
                }
                Err(es) => {
                    $(
                    let got = errors_to_str(&es);
                    assert_eq!(got, $errors);
                    )?
                }
            }
            )*
        }
    }
}

fn errors_to_str(e: &Vec<SetError<ErrorLabel>>) -> String {
    e.iter()
        .map(|e| format!("{}..{}", e.span.start, e.span.end))
        .join(" ")
}

use itertools::Itertools;
use jonla_parser::error::error_printer::ErrorLabel;
use jonla_parser::error::set_error::SetError;
pub(crate) use parse_test;
