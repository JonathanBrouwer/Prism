use crate::lang::env::{DbEnv, UniqueVariableId};
use crate::lang::error::TypeError;
use crate::parser::parse_expr::GrammarEnvEntry;
use crate::parser::{ParsedIndex, ParsedPrismExpr};
use prism_parser::core::allocs::Allocs;
use prism_parser::core::pos::Pos;
use prism_parser::core::span::Span;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parsable::guid::Guid;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::Deref;

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
    (CheckedIndex, DbEnv, HashMap<UniqueVariableId, usize>),
);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum ValueOrigin {
    /// This is an AST node directly from the source code
    SourceCode(Span),
    /// This is the type of another AST node
    TypeOf(CheckedIndex),
    /// This is an AST node generated from expanding the given free variable
    FreeSub(CheckedIndex),
    /// This is an (initally free) AST node generated because type checking a node failed
    Failure,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct CheckedIndex(pub usize);

impl Display for CheckedIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0)
    }
}

impl Deref for CheckedIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone)]
pub enum CheckedPrismExpr<'arn, 'grm: 'arn> {
    // Real expressions
    Free,
    Type,
    Let(CheckedIndex, CheckedIndex),
    DeBruijnIndex(usize),
    FnType(CheckedIndex, CheckedIndex),
    FnConstruct(CheckedIndex),
    FnDestruct(CheckedIndex, CheckedIndex),
    Shift(CheckedIndex, usize),
    TypeAssert(CheckedIndex, CheckedIndex),

    // Temporary expressions after parsing
    GrammarValue(&'arn GrammarFile<'arn, 'grm>, Guid),
    GrammarType,
}

pub struct PrismEnv<'arn, 'grm: 'arn> {
    // Allocs
    pub input: &'grm str,
    pub allocs: Allocs<'arn>,

    // Parsed Values
    pub parsed_values: Vec<ParsedPrismExpr<'arn, 'grm>>,
    pub parsed_spans: Vec<Span>,
    pub grammar_envs: HashMap<Guid, GrammarEnvEntry>,

    // Checked Values
    pub checked_values: Vec<CheckedPrismExpr<'arn, 'grm>>,
    pub checked_origins: Vec<ValueOrigin>,
    checked_types: HashMap<CheckedIndex, CheckedIndex>,

    // State during type checking
    tc_id: usize,
    pub errors: Vec<TypeError>,
    toxic_values: HashSet<CheckedIndex>,
    queued_beq_free: HashMap<CheckedIndex, Vec<QueuedConstraint>>,
}

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn new(allocs: Allocs<'arn>) -> Self {
        Self {
            input: "",
            allocs,

            parsed_values: Default::default(),
            parsed_spans: Default::default(),
            grammar_envs: Default::default(),
            checked_values: Default::default(),
            checked_origins: Default::default(),
            checked_types: Default::default(),
            tc_id: Default::default(),
            errors: Default::default(),
            toxic_values: Default::default(),
            queued_beq_free: Default::default(),
        }
    }

    pub fn store_from_source(&mut self, e: ParsedPrismExpr<'arn, 'grm>, span: Span) -> ParsedIndex {
        self.store_parsed(e, span)
    }

    pub fn store_test(&mut self, e: CheckedPrismExpr<'arn, 'grm>) -> CheckedIndex {
        self.store_checked(
            e,
            ValueOrigin::SourceCode(Span::new(Pos::start(), Pos::start())),
        )
    }

    fn store_parsed(&mut self, e: ParsedPrismExpr<'arn, 'grm>, origin: Span) -> ParsedIndex {
        self.parsed_values.push(e);
        self.parsed_spans.push(origin);
        ParsedIndex(self.parsed_values.len() - 1)
    }

    pub fn store_checked(
        &mut self,
        e: CheckedPrismExpr<'arn, 'grm>,
        origin: ValueOrigin,
    ) -> CheckedIndex {
        self.checked_values.push(e);
        self.checked_origins.push(origin);
        CheckedIndex(self.checked_values.len() - 1)
    }

    pub fn reset(&mut self) {
        self.queued_beq_free.clear();
        self.errors.clear();
        self.toxic_values.clear();
        self.tc_id = 0;
    }

    pub fn erase_arena(self) -> PrismEnv<'grm, 'grm> {
        todo!()
    }
}
