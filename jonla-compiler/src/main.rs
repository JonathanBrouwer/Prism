use jonla_parser::grammar::GrammarFile;
use jonla_parser::parse_grammar;
use jonla_parser::parser::actual::error_printer::*;
use jonla_parser::parser::actual::parser_rule::run_parser_rule;
use jonla_parser::parser::core::stream::StringStream;

fn main() {
    let grammar = include_str!("../resources/grammar");
    let grammar: GrammarFile = match parse_grammar(grammar) {
        Ok(ok) => ok,
        Err(e) => {
            print_set_error(e, "grammar", grammar, false);
            return;
        },
    };



    let filename = "program.jnl";
    let input = include_str!("../resources/program.jnl");
    let result: Result<_, _> = run_parser_rule(&grammar, "block", StringStream::new(input));

    match result {
        Ok(o) => println!("Result: {:?}", o.1.to_string(input)),
        // Err(e) => print_set_error(e, filename, input, false),
        Err(e) => print_tree_error(e, filename, input, true),
    }
}
