use jonla_macros::parser::core::error::display;
use crate::autogen::parse::parse_term;

#[allow(unused)]
#[rustfmt::skip]
mod autogen;

fn main() {
    let input = include_str!("../resources/program.jnl");

    match parse_term(input).collapse() {
        Ok(o) => println!("Result: {:?}", o),
        Err(e) => display(e, input),
    }
}