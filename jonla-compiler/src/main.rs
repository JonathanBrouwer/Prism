#[allow(unused)]
#[rustfmt::skip]
mod autogen;

use crate::autogen::parse::parse_term;

fn main() {
    let input = include_str!("../resources/program.jl");
    let result = parse_term(input);
    println!("{:?}", result);
}
