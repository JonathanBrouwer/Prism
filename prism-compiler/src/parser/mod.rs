use crate::lang::error::PrismError;
use crate::lang::{CoreIndex, PrismDb};
use prism_parser::core::input::Input;
use prism_parser::core::input_table::{InputTable, InputTableIndex};
use prism_parser::core::pos::Pos;
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

pub static GRAMMAR: LazyLock<(InputTable, Arc<GrammarFile>)> = LazyLock::new(|| {
    let (table, grammar) =
        parse_grammar::<SetError>(include_str!("../../resources/prism.pg")).unwrap_or_eprint();
    (table.deep_clone(), grammar)
});

impl PrismDb {
    pub fn load_file(&mut self, path: PathBuf) -> InputTableIndex {
        let program = std::fs::read_to_string(&path).unwrap();
        self.input.get_or_push_file(program, path)
    }

    pub fn load_test(&mut self, data: &str, path_name: &'static str) -> InputTableIndex {
        self.input
            .get_or_push_file(data.to_string(), path_name.into())
    }

    pub fn parse_file(&mut self, file: InputTableIndex) -> ParsedIndex {
        let mut parsables = HashMap::new();
        parsables.insert("Expr", ParsableDyn::new::<ParsedIndex>());

        match run_parser_rule_raw::<PrismDb, SetError>(
            &GRAMMAR.1,
            "expr",
            self.input.clone(),
            file,
            parsables,
            self,
        )
        .map(|v| *v.rtrn.into_value::<ParsedIndex>())
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

#[derive(Clone)]
pub enum ParsedPrismExpr {
    // Real expressions
    Free,
    Type,
    Let(Input, ParsedIndex, ParsedIndex),
    FnType(Input, ParsedIndex, ParsedIndex),
    FnConstruct(Input, ParsedIndex),
    FnDestruct(ParsedIndex, ParsedIndex),
    TypeAssert(ParsedIndex, ParsedIndex),

    // Temporary expressions after parsing
    Name(Input),
    ShiftTo {
        expr: ParsedIndex,
        captured_env: VarMap,
        adapt_env_len: usize,
        grammar: Arc<GrammarFile>,
    },
    GrammarValue(Arc<GrammarFile>),
    GrammarType,
    Include(Input, CoreIndex),
}
