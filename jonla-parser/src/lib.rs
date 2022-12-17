#[macro_use]
extern crate lazy_static;

use grammar::from_action_result::parse_grammarfile;
use grammar::grammar::GrammarFile;
use crate::core::adaptive::GrammarState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::core::stream::StringStream;
use grammar::run::run_parser_rule;

pub mod core;
pub mod grammar;
pub mod error;

lazy_static! {
    pub static ref META_GRAMMAR: GrammarFile<'static> = {
        let meta_grammar = include_str!("../resources/bootstrap.json");
        serde_json::from_str(meta_grammar).unwrap()
    };
    pub static ref META_GRAMMAR_STATE: GrammarState<'static, 'static> =
        GrammarState::new(&META_GRAMMAR);
}

pub fn parse_grammar<'a, E: ParseError<L = ErrorLabel<'a>>>(
    grammar: &'a str,
) -> Result<GrammarFile, Vec<E>> {
    let grammar_stream: StringStream = StringStream::new(grammar);

    run_parser_rule(&META_GRAMMAR, "toplevel", grammar_stream)
        .map(|pr| parse_grammarfile(&pr.1, grammar))
}
