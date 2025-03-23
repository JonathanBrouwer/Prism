use crate::lang::PrismEnv;
use crate::lang::error::TypeError;
use prism_parser::core::allocs::Allocs;
use prism_parser::core::input_table::InputTable;
use prism_parser::core::span::Span;
use prism_parser::error::aggregate_error::{AggregatedParseError, ParseResultExt};
use prism_parser::error::set_error::SetError;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parsable::parsable_dyn::ParsableDyn;
use prism_parser::parse_grammar;
use prism_parser::parser::VarMap;
use prism_parser::parser::parser_instance::run_parser_rule_raw;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::LazyLock;

mod display;
pub mod named_env;
pub mod parse_expr;
mod parsed_to_checked;

pub static GRAMMAR: LazyLock<GrammarFile<'static>> = LazyLock::new(|| {
    *parse_grammar::<SetError>(
        include_str!("../../resources/prism.pg"),
        Allocs::new_leaking(),
    )
    .unwrap_or_eprint()
});

pub fn parse_prism_in_env<'p>(
    program: &'p str,
    env: &mut PrismEnv<'p>,
) -> Result<ParsedIndex, AggregatedParseError<'p, SetError<'p>>> {
    let file = env.input.push_file(program);

    let mut parsables = HashMap::new();
    parsables.insert("Expr", ParsableDyn::new::<ParsedIndex>());

    run_parser_rule_raw::<PrismEnv<'p>, SetError>(
        &GRAMMAR,
        "expr",
        env.input.clone(),
        file,
        env.allocs,
        parsables,
        env,
    )
    .map(|v| *v.into_value())
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ParsedIndex(pub usize);

impl Deref for ParsedIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone)]
pub enum ParsedPrismExpr<'arn> {
    // Real expressions
    Free,
    Type,
    Let(&'arn str, ParsedIndex, ParsedIndex),
    FnType(&'arn str, ParsedIndex, ParsedIndex),
    FnConstruct(&'arn str, ParsedIndex),
    FnDestruct(ParsedIndex, ParsedIndex),
    TypeAssert(ParsedIndex, ParsedIndex),

    // Temporary expressions after parsing
    Name(&'arn str),
    ShiftTo {
        expr: ParsedIndex,
        captured_env: VarMap<'arn>,
        adapt_env_len: usize,
        grammar: &'arn GrammarFile<'arn>,
    },
    GrammarValue(&'arn GrammarFile<'arn>),
    GrammarType,
}

pub struct PrismParseEnv<'arn> {
    // Allocs
    pub input: InputTable<'arn>,
    pub allocs: Allocs<'arn>,

    // Value store
    pub values: Vec<ParsedPrismExpr<'arn>>,
    pub value_origins: Vec<Span>,
    pub errors: Vec<TypeError>,
}
