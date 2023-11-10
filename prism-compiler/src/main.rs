pub mod coc;

use prism_compiler::{GRAMMAR, parse_prism};
use crate::coc::Expr;
use prism_parser::error::error_printer::print_set_error;
use prism_parser::parse_grammar;
use prism_parser::parser::parser_instance::{Arena, run_parser_rule};

fn main() {
    let arena = Arena::new();
    let input = include_str!("../resources/program.pr");
    let Some(expr) = parse_prism(input, &arena) else {
        return
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

