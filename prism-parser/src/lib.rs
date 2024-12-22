use std::sync::LazyLock;

use grammar::from_action_result::parse_grammarfile;

use crate::core::cache::Allocs;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::rule_action::RuleAction;
use crate::grammar::GrammarFile;
use crate::parser::parser_instance::run_parser_rule;

pub mod core;
pub mod error;
pub mod grammar;
pub mod parsable;
pub mod parser;

pub static META_GRAMMAR: LazyLock<GrammarFile<'static, 'static>> = LazyLock::new(|| {
    let meta_grammar = include_bytes!("../resources/bootstrap.bincode");
    bincode::deserialize(meta_grammar).unwrap()
});

pub fn parse_grammar<'grm, E: ParseError<L = ErrorLabel<'grm>>>(
    grammar: &'grm str,
    allocs: Allocs<'grm>,
) -> Result<GrammarFile<'grm, 'grm>, AggregatedParseError<'grm, E>> {
    run_parser_rule(&META_GRAMMAR, "toplevel", grammar, allocs, |ar| {
        parse_grammarfile::<'grm, 'grm>(ar, grammar, allocs, |ar| {
            *ar.into_value::<RuleAction<'grm, 'grm>>()
        })
        .expect("Grammars parsed by the meta grammar should have a legal AST.")
    })
}
