use crate::autogen::parse::parse_test;

mod autogen;

fn main() {
    let input = include_str!("../resources/program");
    let _result = parse_test(input);
    println!("Hello, world!");
}
