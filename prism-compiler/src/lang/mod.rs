use crate::lang::env::{Env, UniqueVariableId};
use std::collections::{HashMap, HashSet};
use prism_parser::core::span::Span;
use crate::lang::error::TypeError;

mod beta_reduce;
mod beta_reduce_head;
pub mod display;
pub mod env;
mod expect_beq;
pub mod from_action_result;
pub mod is_beta_equal;
pub mod simplify;
pub mod type_check;
pub mod error;
mod expect_beq_internal;

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
    queued_beq_free: HashMap<UnionIndex, Vec<((Env, HashMap<UniqueVariableId, usize>), (UnionIndex, Env, HashMap<UniqueVariableId, usize>))>>
    //TODO readd queued_tc: HashMap<UnionIndex, (Env, UnionIndex)>,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum ValueOrigin {
    SourceCode(Span),
    TypeOf(UnionIndex),
    FreeSub(UnionIndex),
    FreeValueFailure(UnionIndex),
    FreeTypeFailure(UnionIndex),
    Test
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct UnionIndex(usize);

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum PartialExpr {
    Type,
    Let(UnionIndex, UnionIndex),
    Var(usize),
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

    pub fn store_test(&mut self, e: PartialExpr) -> UnionIndex {
        self.store(e, ValueOrigin::Test)
    }

    fn store(&mut self, e: PartialExpr, origin: ValueOrigin) -> UnionIndex {
        self.values.push(e);
        self.value_origins.push(origin);
        UnionIndex(self.values.len() - 1)
    }
}