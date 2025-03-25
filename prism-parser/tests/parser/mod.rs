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
macro_rules! parse_test {
    (name: $name:ident syntax: $syntax:literal passing tests: $($input_pass:literal => $expected:literal)* failing tests: $($input_fail:literal $(=> $errors:literal)?)*) => {
        #[allow(unused_imports)]
        #[allow(unused_variables)]
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
            use bumpalo::Bump;
            use prism_parser::core::allocs::Allocs;
            use prism_parser::parsable::parsed::Parsed;
            use prism_parser::parsable::parsable_dyn::ParsableDyn;
            use prism_parser::parsable::action_result::ActionResult;
            use prism_parser::core::input_table::InputTable;

            let syntax: &'static str = $syntax;
            let bump = Bump::new();
            let alloc = Allocs::new(&bump);
            let grammar: &GrammarFile = parse_grammar::<SetError>(syntax, alloc).unwrap_or_eprint();

            let mut parsables = HashMap::new();
            parsables.insert(
                "",
                ParsableDyn::new::<ActionResult>(),
            );

            $({
            let input: &'static str = $input_pass;
            let input_table = Arc::new(InputTable::default());
            let file = input_table.get_or_push_file(input, "test_file".into());
            println!("== Parsing (should be ok): {}", input);


            let got = run_parser_rule_raw::<(), SetError>(&grammar, "start", input_table.clone(), file, alloc, parsables.clone(), &mut ()).unwrap_or_eprint();
            let got = got.to_debug_string(&input_table);
            assert_eq!($expected, got);
            })*

            $({
            let input: &'static str = $input_fail;
            let input_table = Arc::new(InputTable::default());
            let file = input_table.get_or_push_file(input, "test_file".into());
            println!("== Parsing (should be fail): {}", input);

            match run_parser_rule_raw::<(), SetError>(&grammar, "start", input_table.clone(), file, alloc, parsables.clone(), &mut ()) {
                Ok(got) => {
                    let got = got.to_debug_string(&input_table);
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
mod repeat;

mod span_merging;

pub(crate) use parse_test;
