use crate::parser::{GRAMMAR, ParserPrismEnv};
use prism_diags::Diag;
use prism_input::input_table::{InputTable, InputTableIndex};
use prism_input::span::Span;
use prism_parser::core::tokens::Tokens;
use prism_parser::grammar::grammar_file::GrammarFile;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;

pub mod display;
pub mod env;
pub mod error;
pub mod grammar;

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

pub struct PrismDb {
    // File info
    pub input: Arc<InputTable>,
    files: HashMap<InputTableIndex, ProcessedFileTableEntry>,

    // Checked Values
    pub checked_values: Vec<CorePrismExpr>,
    pub checked_origins: Vec<ValueOrigin>,
    pub checked_types: HashMap<CoreIndex, CoreIndex>,

    pub errors: Vec<Diag>,
}

enum ProcessedFileTableEntry {
    Processing,
    Processed(ProcessedFile),
}

#[derive(Clone)]
pub struct ProcessedFile {
    pub core: CoreIndex,
    pub typ: CoreIndex,
    pub tokens: Arc<Tokens>,
}

impl Default for PrismDb {
    fn default() -> Self {
        Self::new()
    }
}

impl PrismDb {
    pub fn new() -> Self {
        Self {
            input: Arc::new(GRAMMAR.0.deep_clone()),

            checked_values: Default::default(),
            checked_origins: Default::default(),
            checked_types: Default::default(),
            errors: Default::default(),
            files: Default::default(),
        }
    }

    pub fn process_file(&mut self, file: InputTableIndex) -> ProcessedFile {
        match self.files.entry(file) {
            Entry::Occupied(v) => match v.get() {
                ProcessedFileTableEntry::Processing => {
                    panic!("Import cycle")
                }
                ProcessedFileTableEntry::Processed(p) => return p.clone(),
            },
            Entry::Vacant(v) => v.insert(ProcessedFileTableEntry::Processing),
        };

        let (core, tokens) = self.parse_prism_file(file);

        let typ = self.type_check(core);
        let processed_file = ProcessedFile { core, typ, tokens };
        self.files.insert(
            file,
            ProcessedFileTableEntry::Processed(processed_file.clone()),
        );
        processed_file
    }

    pub fn parse_prism_file(&mut self, file: InputTableIndex) -> (CoreIndex, Arc<Tokens>) {
        let mut parse_env = ParserPrismEnv::new(self);
        let (parsed, tokens) = parse_env.parse_file(file);
        let core = parse_env.parsed_to_checked(parsed);
        (core, tokens)
    }

    pub fn update_file(&mut self, file: InputTableIndex, content: String) {
        self.files.remove(&file);
        self.input.inner_mut().update_file(file, content);
    }

    pub fn remove_file(&mut self, file: InputTableIndex) {
        self.files.remove(&file);
        self.input.inner_mut().remove(file);
    }

    pub fn store_checked(&mut self, e: CorePrismExpr, origin: ValueOrigin) -> CoreIndex {
        self.checked_values.push(e);
        self.checked_origins.push(origin);
        CoreIndex(self.checked_values.len() - 1)
    }

    pub fn reset(&mut self) {
        self.errors.clear();
    }

    pub fn erase_arena(self) -> PrismDb {
        todo!()
    }
}
