use prism_parser::error::error_printer::print_set_error;
use prism_parser::grammar::grammar_ar::GrammarFile;
use prism_parser::{parse_grammar, run_parser_rule_here};

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
    run_parser_rule_here!(result = &grammar, "block", input);

    match result {
        Ok(o) => println!("Result: {:?}", o.to_string(input)),
        Err(es) => {
            for e in es {
                print_set_error(e, input, false)
            }
        }
    }
}
