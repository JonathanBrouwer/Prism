#[macro_use]
extern crate lazy_static;

use crate::core::adaptive::GrammarState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use grammar::from_action_result::parse_grammarfile;
use grammar::grammar::GrammarFile;
use grammar::parser_instance::run_parser_rule;
use crate::grammar::parser_instance::run_parser_rule_ar;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;

pub mod arena;
pub mod core;
pub mod error;
pub mod grammar;
pub mod rule_action;

lazy_static! {
    pub static ref META_GRAMMAR: GrammarFile<'static, RuleAction<'static>> = {
        let meta_grammar = include_bytes!("../resources/bootstrap.bincode");
        bincode::deserialize(meta_grammar).unwrap()
    };
    pub static ref META_GRAMMAR_STATE: GrammarState<'static, 'static, RuleAction<'static>> =
        GrammarState::new_with(&META_GRAMMAR);
}

pub fn parse_grammar<'grm, E: ParseError<L = ErrorLabel<'grm>>>(
    grammar: &'grm str,
) -> Result<GrammarFile<'grm, RuleAction<'grm>>, Vec<E>> {
    run_parser_rule_ar(&META_GRAMMAR, "toplevel", grammar).map(|pr| {
        parse_grammarfile(&pr, grammar)
            .expect("Grammars parsed by the meta grammar should have a legal AST.")
    })
}
