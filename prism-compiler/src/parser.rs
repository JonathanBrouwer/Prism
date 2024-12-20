use crate::lang::{TcEnv, UnionIndex};
use prism_parser::core::cache::Allocs;
use prism_parser::error::aggregate_error::{AggregatedParseError, ParseResultExt};
use prism_parser::error::set_error::SetError;
use prism_parser::grammar::GrammarFile;
use prism_parser::parse_grammar;
use prism_parser::parser::parser_instance::run_parser_rule;
use std::sync::LazyLock;

pub static GRAMMAR: LazyLock<GrammarFile<'static, 'static>> = LazyLock::new(|| {
    parse_grammar::<SetError>(include_str!("../resources/prism.pg"), Allocs::new_leaking())
        .unwrap_or_eprint()
});

pub fn parse_prism_in_env<'p>(
    program: &'p str,
    env: &mut TcEnv,
) -> Result<UnionIndex, AggregatedParseError<'p, SetError<'p>>> {
    run_parser_rule::<SetError, _>(&GRAMMAR, "expr", program, |r, allocs| {
        env.insert_from_action_result(r, program, allocs)
    })
}

pub fn parse_prism(program: &str) -> Result<(TcEnv, UnionIndex), AggregatedParseError<SetError>> {
    let mut env = TcEnv::default();
    parse_prism_in_env(program, &mut env).map(|i| (env, i))
}
