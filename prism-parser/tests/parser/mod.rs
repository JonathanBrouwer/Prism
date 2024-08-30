mod adaptive;
mod arithmetic;
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
mod span_merging;

const ENABLE_NEW: bool = false;

macro_rules! parse_test {
    (name: $name:ident syntax: $syntax:literal passing tests: $($input_pass:literal => $expected:literal)* failing tests: $($input_fail:literal $(=> $errors:literal)?)*) => {
        mod $name {
            #[allow(unused_imports)]
        #[allow(unused_variables)]
        #[test]
        fn parser1() {
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
            use prism_parser::grammar::rule_action::RuleAction;
            use prism_parser::error::aggregate_error::ParseResultExt;
            use bumpalo::Bump;
            use prism_parser::core::cache::Allocs;

            let syntax: &'static str = $syntax;
            let bump = Bump::new();
            let alloc = Allocs::new(&bump);
            let grammar: GrammarFile = parse_grammar::<SetError>(syntax, alloc).unwrap_or_eprint();

            $({
            let input: &'static str = $input_pass;
            println!("== Parsing (should be ok): {}", input);

            let got = run_parser_rule::<SetError, _>(&grammar, "start", input, |v, _| v.to_string(input)).unwrap_or_eprint();
            assert_eq!($expected, got);
            })*

            $({
            let input: &'static str = $input_fail;
            println!("== Parsing (should be fail): {}", input);

            match run_parser_rule::<SetError, _>(&grammar, "start", input, |v, _| v.to_string(input)) {
                Ok(got) => {
                    println!("Got: {:?}", got);
                    panic!();
                }
                Err(es) => {
                    $(
                    let got = es.errors.iter()
                        .map(|e| format!("{}..{}", e.span.start, e.span.end))
                        .collect::<Vec<_>>()
                        .join(" ");
                    assert_eq!(got, $errors);
                    )?
                }
            }
            })*
        }


            #[allow(unused_imports)]
            #[allow(unused_variables)]
            #[test]
            fn parser2() {
                if !$crate::parser::ENABLE_NEW {
                    return
                }
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
                use prism_parser::grammar::rule_action::RuleAction;
                use prism_parser::error::aggregate_error::ParseResultExt;
                use bumpalo::Bump;
                use prism_parser::core::cache::Allocs;
                use prism_parser::parser2::ParserState;


                let syntax: &'static str = $syntax;
                let bump = Bump::new();
                let alloc = Allocs::new(&bump);
                let grammar: GrammarFile = parse_grammar::<SetError>(syntax, alloc).unwrap_or_eprint();

                $({
                let input: &'static str = $input_pass;
                println!("== Parsing (should be ok): {}", input);

                assert!(ParserState::<SetError>::run_rule(&grammar, "start", alloc, input).is_ok());
                // let got = run_parser_rule::<SetError, _>(&grammar, "start", input, |v, _| v.to_string(input)).unwrap_or_eprint();
                // assert_eq!($expected, got);
                })*

                $({
                let input: &'static str = $input_fail;
                println!("== Parsing (should be fail): {}", input);

                assert!(ParserState::<SetError>::run_rule(&grammar, "start", alloc, input).is_err());

                // match run_parser_rule::<SetError, _>(&grammar, "start", input, |v, _| v.to_string(input)) {
                //     Ok(got) => {
                //         println!("Got: {:?}", got);
                //         panic!();
                //     }
                //     Err(es) => {
                //         $(
                //         let got = es.errors.iter()
                //             .map(|e| format!("{}..{}", e.span.start, e.span.end))
                //             .collect::<Vec<_>>()
                //             .join(" ");
                //         assert_eq!(got, $errors);
                //         )?
                //     }
                // }
                })*
            }
        }
    }
}

pub(crate) use parse_test;
