use crate::lang::env::{Env, UniqueVariableId};
use crate::lang::error::TypeError;
use prism_parser::core::pos::Pos;
use prism_parser::core::span::Span;
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

type QueuedConstraint<'grm> = (
    (Env<'grm>, HashMap<UniqueVariableId, usize>),
    (UnionIndex, Env<'grm>, HashMap<UniqueVariableId, usize>),
);

#[derive(Default)]
pub struct TcEnv<'grm> {
    pub values: Vec<PartialExpr<'grm>>,
    pub value_origins: Vec<ValueOrigin>,
    value_types: HashMap<UnionIndex, UnionIndex>,

    tc_id: usize,
    pub errors: Vec<TypeError>,
    toxic_values: HashSet<UnionIndex>,

    // Queues
    queued_beq_free: HashMap<UnionIndex, Vec<QueuedConstraint<'grm>>>,
    // queued_tc: HashMap<UnionIndex, (Env, UnionIndex)>,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum ValueOrigin {
    /// This is an AST node directly from the source code
    SourceCode(Span),
    /// This is the type of another AST node
    TypeOf(UnionIndex),
    /// This is an AST node generated from expanding the given free variable
    FreeSub(UnionIndex),
    /// This is an (initally free) AST node generated because type checking a node failed
    Failure,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct UnionIndex(pub usize);

impl Display for UnionIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0)
    }
}

impl Deref for UnionIndex {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PartialExpr<'grm> {
    Free,
    Type,
    Let(&'grm str, UnionIndex, UnionIndex),

    DeBruijnIndex(usize),
    Name(&'grm str),

    FnType(&'grm str, UnionIndex, UnionIndex),
    FnConstruct(&'grm str, UnionIndex),
    FnDestruct(UnionIndex, UnionIndex),
    Shift(UnionIndex, usize),
    TypeAssert(UnionIndex, UnionIndex),
}

impl<'grm> TcEnv<'grm> {
    pub fn store_from_source(&mut self, e: PartialExpr<'grm>, span: Span) -> UnionIndex {
        self.store(e, ValueOrigin::SourceCode(span))
    }

    pub fn store_test(&mut self, e: PartialExpr<'grm>) -> UnionIndex {
        self.store(
            e,
            ValueOrigin::SourceCode(Span::new(Pos::start(), Pos::start())),
        )
    }

    fn store(&mut self, e: PartialExpr<'grm>, origin: ValueOrigin) -> UnionIndex {
        self.values.push(e);
        self.value_origins.push(origin);
        UnionIndex(self.values.len() - 1)
    }

    pub fn reset(&mut self) {
        self.queued_beq_free.clear();
        self.errors.clear();
        self.toxic_values.clear();
        self.tc_id = 0;
    }
}
