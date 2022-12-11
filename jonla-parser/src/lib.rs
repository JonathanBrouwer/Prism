#[macro_use]
extern crate lazy_static;

use crate::from_action_result::parse_grammarfile;
use crate::grammar::GrammarFile;
use crate::parser::actual::error_printer::ErrorLabel;
use crate::parser::actual::parser_rule::run_parser_rule;
use crate::parser::core::error::ParseError;
use crate::parser::core::stream::StringStream;

#[allow(clippy::new_without_default)]
pub mod grammar;
pub mod parser;
pub mod from_action_result;

lazy_static! {
    pub static ref META_GRAMMAR: GrammarFile = {
        let meta_grammar = include_str!("../resources/bootstrap.json");
        serde_json::from_str(meta_grammar).unwrap()
    };
}

pub fn parse_grammar<'a, E: ParseError<L = ErrorLabel<'a>>>(grammar: &'a str) -> Result<GrammarFile, E> {
    let grammar_stream: StringStream = StringStream::new(grammar);
    run_parser_rule(&META_GRAMMAR, "toplevel", grammar_stream).map(|pr| parse_grammarfile(&pr.1, grammar))
}
