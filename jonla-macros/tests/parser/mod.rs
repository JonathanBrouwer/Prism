mod layout;
mod list;
mod literal;
mod minor;
mod parser_tests;
mod repeat;
macro_rules! parse_test {
    (name: $name:ident syntax: $syntax:literal passing tests: $($input_pass:literal => $expected:literal)* failing tests: $($input_fail:literal)*) => {
        #[allow(unused_imports)]
        #[test]
        fn $name() {
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

pub(crate) use parse_test;
