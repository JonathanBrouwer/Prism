#![allow(dead_code)]

use prism_parser::error::aggregate_error::ParseResultExt;
use prism_parser::error::set_error::SetError;
use prism_parser::parse_grammar;
use std::fs::File;

fn main() {
    let input = include_str!("../resources/meta.grammar");
    let grammar2 = parse_grammar::<SetError>(input).unwrap_or_eprint();

    let mut file = File::create("prism-parser/resources/bootstrap.json").unwrap();
    serde_json::to_writer_pretty(&mut file, &grammar2).unwrap();
    let mut file = File::create("prism-parser/resources/bootstrap.bincode").unwrap();
    bincode::serialize_into(&mut file, &grammar2).unwrap();
}
