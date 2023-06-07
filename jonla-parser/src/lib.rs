#[macro_use]
extern crate lazy_static;

use crate::core::adaptive::GrammarState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use grammar::from_action_result::parse_grammarfile;
use grammar::grammar::GrammarFile;
use grammar::run::run_parser_rule;

pub mod arena;
pub mod core;
pub mod error;
pub mod grammar;

lazy_static! {
    pub static ref META_GRAMMAR: GrammarFile<'static> = {
        let meta_grammar = include_bytes!("../resources/bootstrap.bincode");
        bincode::deserialize(meta_grammar).unwrap()
    };
    pub static ref META_GRAMMAR_STATE: GrammarState<'static, 'static> =
        GrammarState::new(&META_GRAMMAR);
}

pub fn parse_grammar<'a, E: ParseError<L = ErrorLabel<'a>>>(
    grammar: &'a str,
) -> Result<GrammarFile, Vec<E>> {
    run_parser_rule(&META_GRAMMAR, "toplevel", grammar).map(|pr| {
        parse_grammarfile(&pr.1, grammar)
            .expect("Grammars parsed by the meta grammar should have a legal AST.")
    })
}
