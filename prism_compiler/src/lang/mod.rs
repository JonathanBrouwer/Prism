use crate::args::PrismArgs;
use crate::lang::diags::ErrorGuaranteed;
use prism_diag::Diag;
use prism_input::input_table::{InputTable, InputTableIndex};
use prism_input::span::Span;
use prism_input::tokens::Tokens;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;

pub mod diags;
pub mod display;
pub mod env;
pub mod error;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum ValueOrigin {
    /// This is an AST node directly from the source code
    SourceCode(Span),
    /// This is the type of another AST node
    TypeOf(CoreIndex),
    /// This is an AST node generated from expanding the given free variable
    FreeSub(CoreIndex),
    /// This is an (initially free) AST node generated because type checking a node failed
    Failure(ErrorGuaranteed),
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
    GrammarValue(Arc<()>),
    GrammarType,
}

pub struct PrismDb {
    pub args: PrismArgs,

    // File info
    pub input: Arc<InputTable>,
    files: HashMap<InputTableIndex, ProcessedFileTableEntry>,

    // Checked Values
    pub values: Vec<CorePrismExpr>,
    pub origins: Vec<ValueOrigin>,
    pub checked_types: HashMap<CoreIndex, CoreIndex>,

    pub diags: Vec<Diag>,
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
        Self::new(PrismArgs::default())
    }
}

impl PrismDb {
    pub fn new(args: PrismArgs) -> Self {
        Self {
            args,
            input: Default::default(),
            values: Default::default(),
            origins: Default::default(),
            checked_types: Default::default(),
            diags: Default::default(),
            files: Default::default(),
        }
    }

    pub fn process_main_file(&mut self) -> ProcessedFile {
        let file = self.args.input.clone();
        let file = match self.load_file(file.into()) {
            Ok(file) => file,
            Err(err) => {
                return ProcessedFile {
                    core: self.store(CorePrismExpr::Free, ValueOrigin::Failure(err)),
                    typ: self.store(CorePrismExpr::Free, ValueOrigin::Failure(err)),
                    tokens: Arc::new(Tokens::Multi(vec![])),
                };
            }
        };

        self.process_file(file)
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

    pub fn update_file(&mut self, file: InputTableIndex, content: String) {
        self.files.remove(&file);
        self.input.inner_mut().update_file(file, content);
    }

    pub fn remove_file(&mut self, file: InputTableIndex) {
        self.files.remove(&file);
        self.input.inner_mut().remove(file);
    }

    pub fn store(&mut self, e: CorePrismExpr, origin: ValueOrigin) -> CoreIndex {
        self.values.push(e);
        self.origins.push(origin);
        CoreIndex(self.values.len() - 1)
    }
}
