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
mod repeat;
macro_rules! parse_test {
    (name: $name:ident syntax: $syntax:literal passing tests: $($input_pass:literal => $expected:literal)* failing tests: $($input_fail:literal $(=> $errors:literal)?)*) => {
        #[allow(unused)]
        #[test]
        fn $name() {
            use std::sync::Arc;
            use prism_parser::parser::parser_instance::run_parser_rule_raw;
            use prism_parser::parse_grammar;
            use prism_parser::grammar::grammar_file::GrammarFile;
            use prism_parser::grammar;
            use prism_parser::error::empty_error::EmptyError;
            use prism_parser::core::presult::PResult;
            use prism_parser::core::presult::PResult::*;
            use prism_parser::core::pos::Pos;
            use prism_parser::error::error_printer::*;
            use prism_parser::core::context::ParserContext;
            use std::collections::HashMap;
            use prism_parser::error::set_error::SetError;
            use prism_parser::grammar::rule_action::RuleAction;
            use prism_parser::error::aggregate_error::ParseResultExt;
            use prism_parser::parsable::parsed::Parsed;
            use prism_parser::parsable::parsable_dyn::ParsableDyn;
            use prism_parser::parsable::action_result::ActionResult;
            use prism_parser::core::input_table::InputTable;

            let syntax: &'static str = $syntax;
            let (input_table, grammar, _) = parse_grammar::<SetError>(syntax).unwrap_or_eprint();

            let mut parsables = HashMap::new();
            parsables.insert(
                "",
                ParsableDyn::new::<ActionResult>(),
            );

            let mut counter = 0;
            $({
            let input: &'static str = $input_pass;
            let file = input_table.get_or_push_file(input.to_string(), format!("test_file_ok{counter}").into());
            println!("== Parsing {counter} (should be ok): {}", input);
            counter += 1;


            let got = run_parser_rule_raw::<(), SetError>(&grammar, "start", input_table.clone(), file, parsables.clone(), &mut ()).unwrap_or_eprint().parsed;
            let got = format!("{got:?}");
            assert_eq!($expected, got);
            })*

            let mut counter = 0;
            $({
            let input: &'static str = $input_fail;
            let file = input_table.get_or_push_file(input.to_string(), format!("test_file_err{counter}").into());
            println!("== Parsing {counter} (should be fail): {}", input);
            counter += 1;

            match run_parser_rule_raw::<(), SetError>(&grammar, "start", input_table.clone(), file, parsables.clone(), &mut ()) {
                Ok(got) => {
                    let got = format!("{:?}", got.parsed);
                    println!("Got: {:?}", got);
                    panic!();
                }
                Err(es) => {
                    $(
                    let got = es.errors.iter()
                        .map(|e| format!("{}..{}", e.pos.start, e.pos.end))
                        .collect::<Vec<_>>()
                        .join(" ");
                    assert_eq!(got, $errors);
                    )?
                }
            }
            })*
        }
    }
}

mod span_merging;

pub(crate) use parse_test;
