use jonla_macros::parser::core::error::display;
use crate::autogen::parse::parse_term;

#[allow(unused)]
#[rustfmt::skip]
mod autogen;

fn main() {
    let input = include_str!("../resources/program.jnl");
    let result = parse_term(input);

    let (errs, o) = result.collapse();

    for err in errs {
        display(err, input);
    }
    println!("Result: {:?}", o);
}