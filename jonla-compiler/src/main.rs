use jonla_parser::grammar;
use jonla_parser::grammar::{GrammarFile};
use jonla_parser::parser::actual::error_printer::*;
use jonla_parser::parser::actual::parser_rule::run_parser_rule;
use jonla_parser::parser::core::stream::StringStream;

fn main() {
    let grammar: GrammarFile = match grammar::grammar_def::toplevel(include_str!("../resources/meta.grammar")) {
        Ok(ok) => ok,
        Err(err) => {
            panic!("{}", err);
        }
    };


    let input = include_str!("../resources/grammar");
    let input_stream: StringStream = input.into();
    let result: Result<_, _> = run_parser_rule(&grammar, "toplevel", input_stream);

    match result {
        Ok(o) => println!("Result: {:?}", o.1.to_string(input)),
        Err(e) => print_set_error(e, "", input, false),
    }

    // let filename = "program.jnl";
    // let input = include_str!("../resources/program.jnl");
    // let input_stream: StringStream = input.into();
    // let result: Result<_, _> = run_parser_rule(&grammar, "block", input_stream);
    //
    // match result {
    //     Ok(o) => println!("Result: {:?}", o.1.to_string(input)),
    //     // Err(e) => print_set_error(e, filename, input, false),
    //     Err(e) => print_tree_error(e, filename, input, true),
    // }
}
