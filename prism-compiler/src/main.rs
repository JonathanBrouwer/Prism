pub mod coc;

use crate::coc::Expr;
use prism_parser::error::error_printer::print_set_error;
use prism_parser::parse_grammar;
use prism_parser::parser::parser_instance::run_parser_rule;

fn main() {
    let grammar = include_str!("../resources/grammar");
    let grammar = match parse_grammar(grammar) {
        Ok(ok) => ok,
        Err(es) => {
            for e in es {
                print_set_error(e, grammar, false);
            }
            return;
        }
    };

    let input = include_str!("../resources/program.pr");
    let expr: Result<_, _> = run_parser_rule(&grammar, "block", input, |r| {
        Expr::from_action_result(&r, input)
    });
    let expr = match expr {
        Ok(o) => o,
        Err(es) => {
            for e in es {
                print_set_error(e, input, false)
            }
            return;
        }
    };
    println!("Program:\n{}", &expr);

    // let typ = match tc(&expr, &Env::new()) {
    //     Ok(typ) => typ,
    //     Err(err) => {
    //         println!("Type error:\n{err:?}");
    //         return;
    //     }
    // };
    // println!("Type:\n{typ}");
}
