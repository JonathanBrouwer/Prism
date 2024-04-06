mod adaptive;
mod infinite;
mod lambda;
mod layout;
mod left_recursion;
mod list;
mod literal;
mod lookahead;
mod minor;
mod parametric;
mod parser_tests;
mod recovery;
mod repeat;

macro_rules! parse_test {
    (name: $name:ident syntax: $syntax:literal passing tests: $($input_pass:literal => $expected:literal)* failing tests: $($input_fail:literal $(=> $errors:literal)?)*) => {
        #[allow(unused_imports)]
        #[allow(unused_variables)]
        #[test]
        fn $name() {
            use prism_parser::parser::parser_instance::run_parser_rule;
            use prism_parser::parse_grammar;
            use prism_parser::grammar::GrammarFile;
            use prism_parser::grammar;
            use prism_parser::error::empty_error::EmptyError;
            use prism_parser::core::parser::Parser;
            use prism_parser::core::presult::PResult;
            use prism_parser::core::presult::PResult::*;
            use prism_parser::core::pos::Pos;
            use prism_parser::error::error_printer::*;
            use prism_parser::parser::parser_rule::parser_rule;
            use prism_parser::core::context::ParserContext;
            use std::collections::HashMap;
            use prism_parser::error::set_error::SetError;
            use prism_parser::rule_action::RuleAction;
            use itertools::Itertools;

            let syntax: &'static str = $syntax;
            let grammar: GrammarFile<_> = match parse_grammar::<SetError<_>>(syntax) {
                Ok(ok) => ok,
                Err(es) => {
                    for e in es {
                        print_set_error(e, syntax, true);
                    }
                    panic!("Failed to parse grammar under test.");
                }
            };

            $({
            let input: &'static str = $input_pass;
            println!("== Parsing (should be ok): {}", input);

            match run_parser_rule(&grammar, "start", input, |v| v.to_string(input)) {
                Ok(got) => {
                    assert_eq!($expected, got);
                }
                Err(es) => {
                    for e in es {
                        // print_set_error(e, "tests", input, true);
                        print_tree_error(e, input, true);
                    }
                    panic!();
                }
            }
            })*

            $({
            let input: &'static str = $input_fail;
            println!("== Parsing (should be fail): {}", input);

            match run_parser_rule::<SetError<_>, _>(&grammar, "start", input, |v| v.to_string(input)) {
                Ok(got) => {
                    println!("Got: {:?}", got);
                    panic!();
                }
                Err(es) => {
                    $(
                    let got = es.iter()
                        .map(|e| format!("{}..{}", e.span.start, e.span.end))
                        .join(" ");
                    assert_eq!(got, $errors);
                    )?
                }
            }
            })*
        }
    }
}

pub(crate) use parse_test;
