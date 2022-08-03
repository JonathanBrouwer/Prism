use crate::autogen::parse::parse_term;
use jonla_macros::parser::error_printer::*;

#[allow(unused)]
#[rustfmt::skip]
mod autogen;

fn main() {
    let input = include_str!("../resources/program.jnl");

    match parse_term(input).collapse() {
        Ok(o) => println!("Result: {:?}", o),
        Err(e) => print_set_error(e, input),
    }
}
