use std::sync::LazyLock;
use typed_arena::Arena;

use grammar::from_action_result::parse_grammarfile;

use crate::core::adaptive::GrammarState;
use crate::core::cache::Allocs;
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

pub static META_GRAMMAR: LazyLock<GrammarFile<'static, RuleAction<'static, 'static>>> =
    LazyLock::new(|| {
        let meta_grammar = include_bytes!("../resources/bootstrap.bincode");
        bincode::deserialize(meta_grammar).unwrap()
    });
pub static META_GRAMMAR_STATE: LazyLock<(
    GrammarState<'static, 'static>,
    VarMap<'static, 'static>,
)> = LazyLock::new(|| {
    let alloc: &'static Allocs = Box::leak(Box::new(Allocs {
        alo_grammarfile: Box::leak(Box::new(Arena::new())),
        alo_grammarstate: Box::leak(Box::new(Arena::new())),
        alo_ar: Box::leak(Box::new(Arena::new())),
        alo_varmap: Box::leak(Box::new(Arena::new())),
    }));
    let (g, i) = GrammarState::new_with(&META_GRAMMAR, alloc);
    (g, i)
});

pub fn parse_grammar<'grm, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
    grammar: &'grm str,
) -> Result<GrammarFile<'grm, RuleAction<'grm, 'grm>>, AggregatedParseError<'grm, E>> {
    run_parser_rule(&META_GRAMMAR, "toplevel", grammar, |ar, _| {
        parse_grammarfile(ar, grammar, parse_rule_action)
            .expect("Grammars parsed by the meta grammar should have a legal AST.")
    })
}
