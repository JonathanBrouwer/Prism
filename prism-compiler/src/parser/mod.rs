use crate::lang::PrismEnv;
use crate::lang::error::TypeError;
use crate::parser::parse_expr::{ScopeEnter, reduce_expr};
use prism_parser::core::cache::Allocs;
use prism_parser::core::span::Span;
use prism_parser::error::aggregate_error::{AggregatedParseError, ParseResultExt};
use prism_parser::error::set_error::SetError;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parsable::guid::Guid;
use prism_parser::parsable::parsable_dyn::ParsableDyn;
use prism_parser::parsable::parsed::Parsed;
use prism_parser::parse_grammar;
use prism_parser::parser::parser_instance::run_parser_rule_raw;
use prism_parser::parser::var_map::VarMap;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::LazyLock;

pub mod parse_expr;
mod parsed_to_checked;

pub static GRAMMAR: LazyLock<GrammarFile<'static, 'static>> = LazyLock::new(|| {
    *parse_grammar::<SetError>(
        include_str!("../../resources/prism.pg"),
        Allocs::new_leaking(),
    )
    .unwrap_or_eprint()
});

pub fn parse_prism_in_env<'p>(
    program: &'p str,
    env: &mut PrismEnv<'_, 'p>,
) -> Result<ParsedIndex, AggregatedParseError<'p, SetError<'p>>> {
    env.input = program;

    let mut parsables = HashMap::new();
    parsables.insert("Expr", ParsableDyn::new::<ParsedIndex>());
    parsables.insert("ScopeEnter", ParsableDyn::new::<ScopeEnter>());

    run_parser_rule_raw::<PrismEnv<'_, 'p>, SetError>(
        &GRAMMAR, "expr", program, env.allocs, parsables, env,
    )
    .map(|v| *reduce_expr(v, env).into_value())
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ParsedIndex(pub usize);

impl Deref for ParsedIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ParsedPrismExpr<'arn, 'grm: 'arn> {
    // Real expressions
    Free,
    Type,
    Let(&'grm str, ParsedIndex, ParsedIndex),
    FnType(&'grm str, ParsedIndex, ParsedIndex),
    FnConstruct(&'grm str, ParsedIndex),
    FnDestruct(ParsedIndex, ParsedIndex),
    TypeAssert(ParsedIndex, ParsedIndex),

    // Temporary expressions after parsing
    Name(&'grm str),
    ShiftLabel(ParsedIndex, Guid),
    ShiftTo(ParsedIndex, Guid, VarMap<'arn, 'grm>),
    ParserValue(Parsed<'arn, 'grm>),
    ParsedType,
}

pub struct PrismParseEnv<'arn, 'grm: 'arn> {
    // Allocs
    pub input: &'grm str,
    pub allocs: Allocs<'arn>,

    // Value store
    pub values: Vec<ParsedPrismExpr<'arn, 'grm>>,
    pub value_origins: Vec<Span>,
    pub errors: Vec<TypeError>,
}
