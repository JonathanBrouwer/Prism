use crate::lang::error::TypeError;
use crate::lang::{CoreIndex, PrismEnv};
use prism_parser::core::allocs::Allocs;
use prism_parser::core::input_table::{InputTable, InputTableIndex};
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
use std::path::{Path, PathBuf};
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

impl<'arn> PrismEnv<'arn> {
    pub fn load_file(&mut self, path: PathBuf) -> InputTableIndex {
        let program = std::fs::read_to_string(&path).unwrap();
        let program = self.allocs.alloc_str(&program);
        self.input.push_file(program, path)
    }

    pub fn load_test(&mut self, data: &'arn str, path_name: &'static str) -> InputTableIndex {
        self.input.push_file(data, path_name.into())
    }

    pub fn parse_file(
        &mut self,
        file: InputTableIndex,
    ) -> Result<ParsedIndex, AggregatedParseError<'arn, SetError<'arn>>> {
        let mut parsables = HashMap::new();
        parsables.insert("Expr", ParsableDyn::new::<ParsedIndex>());

        run_parser_rule_raw::<PrismEnv<'arn>, SetError>(
            &GRAMMAR,
            "expr",
            self.input.clone(),
            file,
            self.allocs,
            parsables,
            self,
        )
        .map(|v| *v.into_value())
    }
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
    Include(&'arn str, CoreIndex),
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
