#![feature(let_chains)]

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::core::cache::Allocs;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::parser::parser_instance::run_parser_rule;
use grammar::grammar_file::GrammarFile;

pub mod core;
pub mod error;
pub mod grammar;
pub mod parsable;
pub mod parser;

pub static META_GRAMMAR: LazyLock<GrammarFile<'static, 'static>> = LazyLock::new(|| {
    let meta_grammar = include_bytes!("../resources/bootstrap.msgpack");
    rmp_serde::decode::from_slice(meta_grammar).unwrap()
});

pub fn parse_grammar<'grm, E: ParseError<L = ErrorLabel<'grm>>>(
    grammar: &'grm str,
    allocs: Allocs<'grm>,
) -> Result<&'grm GrammarFile<'grm, 'grm>, AggregatedParseError<'grm, E>> {
    run_parser_rule::<(), GrammarFile<'grm, 'grm>, E>(
        &META_GRAMMAR,
        "toplevel",
        grammar,
        allocs,
        HashMap::new(),
        &mut (),
    )
}
