use std::collections::HashMap;
use jonla_macros::grammar;
use jonla_macros::grammar::{GrammarFile, RuleBodyExpr};
use jonla_macros::parser::actual::error_printer::*;
use jonla_macros::parser::actual::parser_rule::run_parser_rule;
use jonla_macros::parser::core::stream::StringStream;

fn main() {
    let grammar: GrammarFile = match grammar::grammar_def::toplevel(include_str!("../resources/grammar")) {
        Ok(ok) => ok,
        Err(err) => {
            panic!("{}", err);
        }
    };
    let rules: HashMap<&str, RuleBodyExpr> = grammar.rules.iter().map(|r| (r.name, r.body.clone())).collect();

    let filename = "program.jnl";
    let input = include_str!("../resources/program.jnl");
    let input_stream: StringStream = input.into();
    let result: Result<_, _> = run_parser_rule(&rules, "block", input_stream);








    match result {
        Ok(o) => println!("Result: {:?}", o.1.to_string(input)),
        // Err(e) => print_set_error(e, filename, input, false),
        Err(e) => print_tree_error(e, filename, input, true),
    }
}
