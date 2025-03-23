use crate::lang::env::{DbEnv, UniqueVariableId};
use crate::lang::error::TypeError;
use crate::parser::{ParsedIndex, ParsedPrismExpr};
use prism_parser::core::allocs::Allocs;
use prism_parser::core::input_table::InputTable;
use prism_parser::core::span::Span;
use prism_parser::grammar::grammar_file::GrammarFile;
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

type QueuedConstraint<'arn> = (
    (DbEnv<'arn>, HashMap<UniqueVariableId, usize>),
    (CoreIndex, DbEnv<'arn>, HashMap<UniqueVariableId, usize>),
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

#[derive(Copy, Clone)]
pub enum CorePrismExpr<'arn> {
    Free,
    Type,
    Let(CoreIndex, CoreIndex),
    DeBruijnIndex(usize),
    FnType(CoreIndex, CoreIndex),
    FnConstruct(CoreIndex),
    FnDestruct(CoreIndex, CoreIndex),
    Shift(CoreIndex, usize),
    TypeAssert(CoreIndex, CoreIndex),
    GrammarValue(&'arn GrammarFile<'arn>),
    GrammarType,
}

pub struct PrismEnv<'arn> {
    // Allocs
    pub input: Arc<InputTable<'arn>>,
    pub allocs: Allocs<'arn>,

    // Parsed Values
    pub parsed_values: Vec<ParsedPrismExpr<'arn>>,
    pub parsed_spans: Vec<Span>,

    // Checked Values
    pub checked_values: Vec<CorePrismExpr<'arn>>,
    pub checked_origins: Vec<ValueOrigin>,
    checked_types: HashMap<CoreIndex, CoreIndex>,

    // State during type checking
    tc_id: usize,
    pub errors: Vec<TypeError>,
    toxic_values: HashSet<CoreIndex>,
    queued_beq_free: HashMap<CoreIndex, Vec<QueuedConstraint<'arn>>>,
}

impl<'arn> PrismEnv<'arn> {
    pub fn new(allocs: Allocs<'arn>) -> Self {
        Self {
            input: Arc::new(InputTable::default()),
            allocs,

            parsed_values: Default::default(),
            parsed_spans: Default::default(),
            checked_values: Default::default(),
            checked_origins: Default::default(),
            checked_types: Default::default(),
            tc_id: Default::default(),
            errors: Default::default(),
            toxic_values: Default::default(),
            queued_beq_free: Default::default(),
        }
    }

    pub fn store_from_source(&mut self, e: ParsedPrismExpr<'arn>, span: Span) -> ParsedIndex {
        self.store_parsed(e, span)
    }

    // pub fn store_test(&mut self, e: CorePrismExpr<'arn>) -> CoreIndex {
    //     self.store_checked(
    //         e,
    //         ValueOrigin::SourceCode(Span::new())
    //     )
    // }

    fn store_parsed(&mut self, e: ParsedPrismExpr<'arn>, origin: Span) -> ParsedIndex {
        self.parsed_values.push(e);
        self.parsed_spans.push(origin);
        ParsedIndex(self.parsed_values.len() - 1)
    }

    pub fn store_checked(&mut self, e: CorePrismExpr<'arn>, origin: ValueOrigin) -> CoreIndex {
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

    pub fn erase_arena(self) -> PrismEnv<'arn> {
        todo!()
    }
}
