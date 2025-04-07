#![feature(substr_range)]
#![feature(let_chains)]
#![allow(clippy::too_many_arguments)]

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use self::core::tokens::Tokens;
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

pub static META_GRAMMAR: LazyLock<GrammarFile> = LazyLock::new(|| {
    let meta_grammar = include_bytes!("../resources/bootstrap.msgpack");
    rmp_serde::decode::from_slice(meta_grammar).unwrap()
});

pub fn parse_grammar<E: ParseError<L = ErrorLabel>>(
    grammar: &str,
) -> (
    Arc<InputTable>,
    Arc<GrammarFile>,
    Arc<Tokens>,
    AggregatedParseError<E>,
) {
    let input_table = Arc::new(InputTable::default());
    let file = input_table.get_or_push_file(grammar.into(), "$GRAMMAR$".into());
    let (grammar, tokens, errs) = run_parser_rule::<(), GrammarFile, E>(
        &META_GRAMMAR,
        "toplevel",
        input_table.clone(),
        file,
        HashMap::new(),
        &mut (),
    );
    (input_table, grammar, tokens, errs)
}
