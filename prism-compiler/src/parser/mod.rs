use crate::lang::error::{PrismError, TypeError};
use crate::lang::{CoreIndex, PrismEnv};
use prism_parser::core::allocs::Allocs;
use prism_parser::core::input_table::{InputTable, InputTableIndex};
use prism_parser::core::pos::Pos;
use prism_parser::core::span::Span;
use prism_parser::error::aggregate_error::ParseResultExt;
use prism_parser::error::set_error::SetError;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parsable::parsable_dyn::ParsableDyn;
use prism_parser::parse_grammar;
use prism_parser::parser::VarMap;
use prism_parser::parser::parser_instance::run_parser_rule_raw;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

mod display;
pub mod named_env;
pub mod parse_expr;
mod parsed_to_checked;

pub static GRAMMAR: LazyLock<(InputTable<'static>, &'static GrammarFile<'static>)> =
    LazyLock::new(|| {
        let (table, grammar) = parse_grammar::<SetError>(
            include_str!("../../resources/prism.pg"),
            Allocs::new_leaking(),
        )
        .unwrap_or_eprint();
        (table.deep_clone(), grammar)
    });

impl<'arn> PrismEnv<'arn> {
    pub fn load_file(&mut self, path: PathBuf) -> InputTableIndex {
        let program = std::fs::read_to_string(&path).unwrap();
        let program = self.allocs.alloc_str(&program);
        self.input.get_or_push_file(program, path)
    }

    pub fn load_test(&mut self, data: &'arn str, path_name: &'static str) -> InputTableIndex {
        self.input.get_or_push_file(data, path_name.into())
    }

    pub fn parse_file(&mut self, file: InputTableIndex) -> ParsedIndex {
        let mut parsables = HashMap::new();
        parsables.insert("Expr", ParsableDyn::new::<ParsedIndex>());

        match run_parser_rule_raw::<PrismEnv<'arn>, SetError>(
            &GRAMMAR.1,
            "expr",
            self.input.clone(),
            file,
            self.allocs,
            parsables,
            self,
        )
        .map(|v| *v.into_value::<ParsedIndex>())
        {
            Ok(v) => v,
            Err(es) => {
                for e in es.errors {
                    self.errors.push(PrismError::ParseError(e));
                }
                let placeholder_span = Pos::start_of(file).span_to(Pos::start_of(file));
                self.store_from_source(ParsedPrismExpr::Free, placeholder_span)
            }
        }
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
