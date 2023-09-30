use jonla_parser::error::error_printer::print_set_error;
use jonla_parser::grammar::parser_instance::run_parser_rule;
use jonla_parser::grammar::GrammarFile;
use jonla_parser::parse_grammar;

fn main() {
    let grammar = include_str!("../resources/grammar");
    let grammar: GrammarFile = match parse_grammar(grammar) {
        Ok(ok) => ok,
        Err(es) => {
            for e in es {
                print_set_error(e, grammar, false);
            }
            return;
        }
    };

    let input = include_str!("../resources/program.jnl");
    let result: Result<_, _> = run_parser_rule(&grammar, "block", input);

    match result {
        Ok(o) => println!("Result: {:?}", o.to_string(input)),
        Err(es) => {
            for e in es {
                print_set_error(e, input, false)
            }
        }
    }
}
