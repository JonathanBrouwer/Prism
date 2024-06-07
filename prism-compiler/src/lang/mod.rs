use crate::lang::env::{Env, UniqueVariableId};
use crate::lang::error::TypeError;
use prism_parser::core::pos::Pos;
use prism_parser::core::span::Span;
use std::collections::{HashMap, HashSet};

mod beta_reduce;
mod beta_reduce_head;
pub mod display;
pub mod env;
pub mod error;
mod expect_beq;
mod expect_beq_internal;
pub mod from_action_result;
pub mod is_beta_equal;
pub mod simplify;
pub mod type_check;

type QueuedConstraint = (
    (Env, HashMap<UniqueVariableId, usize>),
    (UnionIndex, Env, HashMap<UniqueVariableId, usize>),
);

#[derive(Default)]
pub struct TcEnv {
    // uf: UnionFind,
    values: Vec<PartialExpr>,
    value_origins: Vec<ValueOrigin>,
    value_types: HashMap<UnionIndex, UnionIndex>,

    tc_id: usize,
    pub errors: Vec<TypeError>,
    toxic_values: HashSet<UnionIndex>,

    // Queues
    queued_beq_free: HashMap<UnionIndex, Vec<QueuedConstraint>>,
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
pub struct UnionIndex(usize);

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum PartialExpr {
    Type,
    Let(UnionIndex, UnionIndex),
    DeBruijnIndex(usize),
    FnType(UnionIndex, UnionIndex),
    FnConstruct(UnionIndex, UnionIndex),
    FnDestruct(UnionIndex, UnionIndex),
    Free,
    Shift(UnionIndex, usize),
}

impl TcEnv {
    pub fn store_from_source(&mut self, e: PartialExpr, span: Span) -> UnionIndex {
        self.store(e, ValueOrigin::SourceCode(span))
    }

    fn store(&mut self, e: PartialExpr, origin: ValueOrigin) -> UnionIndex {
        self.values.push(e);
        self.value_origins.push(origin);
        UnionIndex(self.values.len() - 1)
    }

    pub fn store_test(&mut self, e: PartialExpr) -> UnionIndex {
        self.store(
            e,
            ValueOrigin::SourceCode(Span::new(Pos::start(), Pos::start())),
        )
    }
}
