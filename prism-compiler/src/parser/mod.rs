use crate::lang::{TcEnv, UnionIndex};
use crate::parser::parse_env::ParsedEnv;
use crate::parser::parse_expr::{reduce_expr, ScopeEnter};
use bumpalo::Bump;
use prism_parser::core::cache::Allocs;
use prism_parser::error::aggregate_error::{AggregatedParseError, ParseResultExt};
use prism_parser::error::set_error::SetError;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parsable::parsable_dyn::ParsableDyn;
use prism_parser::parse_grammar;
use prism_parser::parser::parser_instance::run_parser_rule_raw;
use std::collections::HashMap;
use std::sync::LazyLock;

mod parse_env;
pub mod parse_expr;

pub static GRAMMAR: LazyLock<GrammarFile<'static, 'static>> = LazyLock::new(|| {
    *parse_grammar::<SetError>(
        include_str!("../../resources/prism.pg"),
        Allocs::new_leaking(),
    )
    .unwrap_or_eprint()
});

pub fn parse_prism_in_env<'p>(
    program: &'p str,
    env: &mut TcEnv,
) -> Result<UnionIndex, AggregatedParseError<'p, SetError<'p>>> {
    let bump = Bump::new();
    let allocs = Allocs::new(&bump);
    let mut parsables = HashMap::new();
    parsables.insert("Expr", ParsableDyn::new::<UnionIndex>());
    parsables.insert("Env", ParsableDyn::new::<ParsedEnv>());
    parsables.insert("ScopeEnter", ParsableDyn::new::<ScopeEnter>());

    run_parser_rule_raw::<TcEnv, SetError>(&GRAMMAR, "expr", program, allocs, parsables, env)
        .map(|v| *reduce_expr(v, env, allocs).into_value())
}

pub fn parse_prism(program: &str) -> Result<(TcEnv, UnionIndex), AggregatedParseError<SetError>> {
    let mut env = TcEnv::default();
    parse_prism_in_env(program, &mut env).map(|i| (env, i))
}
