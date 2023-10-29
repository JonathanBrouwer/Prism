#[macro_use]
extern crate lazy_static;

use crate::core::adaptive::GrammarState;
use crate::core::adaptive::RuleId;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use grammar::from_action_result::parse_grammarfile;
use grammar::grammar_ar::GrammarFile;
use std::collections::HashMap;
use std::mem;
use crate::parser::parser_instance::run_parser_rule;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;

pub mod core;
pub mod error;
pub mod grammar;
pub mod rule_action;
pub mod parser;

lazy_static! {
    pub static ref META_GRAMMAR: GrammarFile<'static> = {
        let meta_grammar = include_bytes!("../resources/bootstrap.bincode");
        bincode::deserialize(meta_grammar).unwrap()
    };
    pub static ref META_GRAMMAR_STATE: (
        GrammarState<'static, 'static>,
        HashMap<&'static str, RuleId>
    ) = {
        let (g, i) = GrammarState::new_with(&META_GRAMMAR);
        (g, i.collect())
    };
}

pub fn parse_grammar<'grm, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
    grammar: &'grm str,
) -> Result<GrammarFile<'grm>, Vec<E>> {
    run_parser_rule(&META_GRAMMAR, "toplevel", grammar).map(|pr| {
        parse_ra_grammarfile(&pr, grammar)
            .expect("Grammars parsed by the meta grammar should have a legal AST.")
    })
}

pub fn parse_ra_grammarfile<'b, 'grm>(
    r: &'b ActionResult<'grm>,
    src: &'grm str,
) -> Option<grammar::GrammarFile<'grm, 'grm, RuleAction<'grm>>> {
    let g: grammar::GrammarFile<'b, 'grm, RuleAction<'grm>> = parse_grammarfile(r, src)?;
    //TODO this unsafe should not be necessary
    unsafe { mem::transmute(g) }
}