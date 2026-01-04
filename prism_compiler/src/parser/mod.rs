use crate::lang::{CoreIndex, PrismDb};
use prism_input::input::Input;
use prism_input::input_table::{InputTable, InputTableIndex};
use prism_input::span::Span;
use prism_parser::core::tokens::Tokens;
use prism_parser::error::ParseError;
use prism_parser::error::set_error::SetError;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parsable::parsable_dyn::ParsableDyn;
use prism_parser::parse_grammar;
use prism_parser::parser::VarMap;
use prism_parser::parser::instance::run_parser_rule;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

mod display;
pub mod named_env;
pub mod parse_expr;
mod parsed_to_checked;

pub static GRAMMAR: LazyLock<(InputTable, Arc<GrammarFile>)> = LazyLock::new(|| {
    let (table, grammar, _tokens, errs) =
        parse_grammar::<SetError>(include_str!("../../resources/prism.pg"));
    errs.unwrap_or_eprint(&table);
    (table.deep_clone(), grammar)
});

impl PrismDb {
    pub fn load_main_file(&mut self) -> InputTableIndex {
        let file = self.args.input.clone();
        self.load_file(file.into())
    }

    pub fn load_file(&mut self, path: PathBuf) -> InputTableIndex {
        let program = std::fs::read_to_string(&path).unwrap();
        self.load_input(program, path)
    }

    pub fn load_input(&mut self, data: String, path: PathBuf) -> InputTableIndex {
        self.input.inner_mut().get_or_push_file(data, path)
    }
}

pub struct ParserPrismEnv<'a> {
    db: &'a mut PrismDb,

    // Parsed Values
    pub parsed_values: Vec<ParsedPrismExpr>,
    pub parsed_spans: Vec<Span>,
}

impl<'a> ParserPrismEnv<'a> {
    pub fn new(db: &'a mut PrismDb) -> Self {
        Self {
            db,
            parsed_values: Default::default(),
            parsed_spans: Default::default(),
        }
    }

    pub fn store_from_source(&mut self, e: ParsedPrismExpr, span: Span) -> ParsedIndex {
        self.store_parsed(e, span)
    }

    fn store_parsed(&mut self, e: ParsedPrismExpr, origin: Span) -> ParsedIndex {
        self.parsed_values.push(e);
        self.parsed_spans.push(origin);
        ParsedIndex(self.parsed_values.len() - 1)
    }

    pub fn parse_file(&mut self, file: InputTableIndex) -> (ParsedIndex, Arc<Tokens>) {
        let mut parsables = HashMap::new();
        parsables.insert("Expr", ParsableDyn::new::<ParsedIndex>());

        let (expr, tokens, errs) = run_parser_rule::<ParserPrismEnv<'a>, ParsedIndex, SetError>(
            &GRAMMAR.1,
            "expr",
            self.db.input.clone(),
            file,
            parsables,
            self,
        );

        for err in errs.errors {
            self.db.diags.push(err.diag());
        }

        (*expr, tokens)
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
