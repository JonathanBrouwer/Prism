#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;
use std::sync::LazyLock;

use self::core::allocs::Allocs;
use crate::error::ParseError;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::parser::parser_instance::run_parser_rule;
use grammar::grammar_file::GrammarFile;

pub mod core;
pub mod env;
pub mod error;
pub mod grammar;
pub mod parsable;
pub mod parser;

pub static META_GRAMMAR: LazyLock<GrammarFile<'static>> = LazyLock::new(|| {
    let meta_grammar = include_bytes!("../resources/bootstrap.msgpack");
    rmp_serde::decode::from_slice(meta_grammar).unwrap()
});

pub fn parse_grammar<'arn, E: ParseError<L = ErrorLabel<'arn>>>(
    grammar: &'arn str,
    allocs: Allocs<'arn>,
) -> Result<&'arn GrammarFile<'arn>, AggregatedParseError<'arn, E>> {
    run_parser_rule::<(), GrammarFile<'arn>, E>(
        &META_GRAMMAR,
        "toplevel",
        grammar,
        allocs,
        HashMap::new(),
        &mut (),
    )
}
