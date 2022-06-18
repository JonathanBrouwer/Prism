use crate::autogen::parse::*;

mod autogen;

fn main() {
    let input = include_str!("../resources/program");
    let result = parse_identifier(input);
    println!("{:?}", result.result);
}
