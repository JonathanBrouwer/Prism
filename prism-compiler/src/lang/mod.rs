use crate::lang::env::{DbEnv, UniqueVariableId};
use crate::lang::error::PrismError;
use crate::parser::{GRAMMAR, ParsedIndex, ParsedPrismExpr};
use prism_parser::core::input_table::{InputTable, InputTableIndex};
use prism_parser::core::span::Span;
use prism_parser::grammar::grammar_file::GrammarFile;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;

mod beta_reduce;
mod beta_reduce_head;
pub mod display;
pub mod env;
pub mod error;
mod expect_beq;
mod expect_beq_internal;
pub mod is_beta_equal;
pub mod simplify;
pub mod type_check;

type QueuedConstraint = (
    (DbEnv, HashMap<UniqueVariableId, usize>),
    (CoreIndex, DbEnv, HashMap<UniqueVariableId, usize>),
);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum ValueOrigin {
    /// This is an AST node directly from the source code
    SourceCode(Span),
    /// This is the type of another AST node
    TypeOf(CoreIndex),
    /// This is an AST node generated from expanding the given free variable
    FreeSub(CoreIndex),
    /// This is an (initially free) AST node generated because type checking a node failed
    Failure,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct CoreIndex(pub usize);

impl Display for CoreIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0)
    }
}

impl Deref for CoreIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub enum CorePrismExpr {
    Free,
    Type,
    Let(CoreIndex, CoreIndex),
    DeBruijnIndex(usize),
    FnType(CoreIndex, CoreIndex),
    FnConstruct(CoreIndex),
    FnDestruct(CoreIndex, CoreIndex),
    Shift(CoreIndex, usize),
    TypeAssert(CoreIndex, CoreIndex),
    GrammarValue(Arc<GrammarFile>),
    GrammarType,
}

pub struct PrismEnv {
    // File info
    pub input: Arc<InputTable>,
    files: HashMap<InputTableIndex, ProcessedFileTableEntry>,

    // Parsed Values
    pub parsed_values: Vec<ParsedPrismExpr>,
    pub parsed_spans: Vec<Span>,

    // Checked Values
    pub checked_roots: HashMap<InputTableIndex, CoreIndex>,
    pub checked_values: Vec<CorePrismExpr>,
    pub checked_origins: Vec<ValueOrigin>,
    checked_types: HashMap<CoreIndex, CoreIndex>,

    // State during type checking
    tc_id: usize,
    toxic_values: HashSet<CoreIndex>,
    queued_beq_free: HashMap<CoreIndex, Vec<QueuedConstraint>>,

    pub errors: Vec<PrismError>,
}

enum ProcessedFileTableEntry {
    Processing,
    Processed(ProcessedFile),
}

#[derive(Copy, Clone)]
pub struct ProcessedFile {
    pub parsed: ParsedIndex,
    pub core: CoreIndex,
    pub typ: CoreIndex,
}

impl PrismEnv {
    pub fn new() -> Self {
        Self {
            input: Arc::new(GRAMMAR.0.deep_clone()),

            parsed_values: Default::default(),
            parsed_spans: Default::default(),
            checked_roots: Default::default(),
            checked_values: Default::default(),
            checked_origins: Default::default(),
            checked_types: Default::default(),
            tc_id: Default::default(),
            errors: Default::default(),
            toxic_values: Default::default(),
            queued_beq_free: Default::default(),
            files: Default::default(),
        }
    }

    pub fn process_file(&mut self, file: InputTableIndex) -> ProcessedFile {
        match self.files.entry(file) {
            Entry::Occupied(v) => match v.get() {
                ProcessedFileTableEntry::Processing => {
                    panic!("Import cycle")
                }
                ProcessedFileTableEntry::Processed(p) => return *p,
            },
            Entry::Vacant(v) => v.insert(ProcessedFileTableEntry::Processing),
        };

        let parsed = self.parse_file(file);
        let core = self.parsed_to_checked(parsed);
        let typ = self.type_check(core);
        let processed_file = ProcessedFile { parsed, core, typ };
        self.files
            .insert(file, ProcessedFileTableEntry::Processed(processed_file));
        processed_file
    }

    pub fn store_from_source(&mut self, e: ParsedPrismExpr, span: Span) -> ParsedIndex {
        self.store_parsed(e, span)
    }

    fn store_parsed(&mut self, e: ParsedPrismExpr, origin: Span) -> ParsedIndex {
        self.parsed_values.push(e);
        self.parsed_spans.push(origin);
        ParsedIndex(self.parsed_values.len() - 1)
    }

    pub fn store_checked(&mut self, e: CorePrismExpr, origin: ValueOrigin) -> CoreIndex {
        self.checked_values.push(e);
        self.checked_origins.push(origin);
        CoreIndex(self.checked_values.len() - 1)
    }

    pub fn reset(&mut self) {
        self.queued_beq_free.clear();
        self.errors.clear();
        self.toxic_values.clear();
        self.tc_id = 0;
    }

    pub fn erase_arena(self) -> PrismEnv {
        todo!()
    }
}
