#![feature(substr_range)]
#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use self::core::allocs::Allocs;
use crate::core::input_table::InputTable;
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

pub const META_GRAMMAR_STR: &str = include_str!("../resources/meta.pg");
pub static META_GRAMMAR: LazyLock<GrammarFile<'static>> = LazyLock::new(|| {
    let meta_grammar = include_bytes!("../resources/bootstrap.msgpack");
    rmp_serde::decode::from_slice(meta_grammar).unwrap()
});

pub fn parse_grammar<'arn, E: ParseError<L = ErrorLabel>>(
    grammar: &'arn str,
    allocs: Allocs<'arn>,
) -> Result<(Arc<InputTable<'arn>>, &'arn GrammarFile<'arn>), AggregatedParseError<'arn, E>> {
    let input_table = Arc::new(InputTable::default());
    let file = input_table.get_or_push_file(grammar, "$GRAMMAR$".into());
    run_parser_rule::<(), GrammarFile<'arn>, E>(
        &META_GRAMMAR,
        "toplevel",
        input_table.clone(),
        file,
        allocs,
        HashMap::new(),
        &mut (),
    )
    .map(|v| (input_table, v))
}
