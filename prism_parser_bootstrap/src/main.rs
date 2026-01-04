use prism_parser::error::set_error::SetError;
use prism_parser::parse_grammar;
use std::fs::File;

pub const META_GRAMMAR_STR: &str = include_str!("../../prism_parser/resources/meta.pg");

fn main() {
    let (table, grammar2, _, errs) = parse_grammar::<SetError>(META_GRAMMAR_STR);
    errs.unwrap_or_eprint(&table);

    let mut file = File::create("./prism_parser/resources/bootstrap.json").unwrap();
    serde_json::to_writer_pretty(&mut file, &grammar2).unwrap();
    let mut file = File::create("./prism_parser/resources/bootstrap.msgpack").unwrap();
    rmp_serde::encode::write_named(&mut file, &grammar2).unwrap();
}
