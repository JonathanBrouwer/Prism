#[macro_use]
extern crate lazy_static;

use typed_arena::Arena;

use grammar::from_action_result::parse_grammarfile;

use crate::core::adaptive::GrammarState;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::GrammarFile;
use crate::parser::parser_instance::run_parser_rule;
use crate::parser::var_map::VarMap;
use crate::rule_action::from_action_result::parse_rule_action;
use crate::rule_action::RuleAction;

pub mod core;
pub mod error;
pub mod grammar;
pub mod parser;
pub mod rule_action;

lazy_static! {
    pub static ref META_GRAMMAR: GrammarFile<'static, RuleAction<'static, 'static>> = {
        let meta_grammar = include_bytes!("../resources/bootstrap.bincode");
        bincode::deserialize(meta_grammar).unwrap()
    };
    pub static ref META_GRAMMAR_STATE: (GrammarState<'static, 'static>, VarMap<'static, 'static>) = {
        let alloc: &'static Arena<_> = Box::leak(Box::new(Arena::new()));
        let (g, i) = GrammarState::new_with(&META_GRAMMAR, alloc);
        (g, i)
    };
}

pub fn parse_grammar<'grm, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
    grammar: &'grm str,
) -> Result<GrammarFile<'grm, RuleAction<'grm, 'grm>>, AggregatedParseError<'grm, E>> {
    run_parser_rule(&META_GRAMMAR, "toplevel", grammar, |ar| {
        parse_grammarfile(ar, grammar, parse_rule_action)
            .expect("Grammars parsed by the meta grammar should have a legal AST.")
    })
}
