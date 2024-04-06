use crate::coc::env::{Env, UniqueVariableId};
use crate::coc::type_check::TcError;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

mod beta_reduce;
mod beta_reduce_head;
mod display;
pub mod env;
pub mod exhaustive;
mod expect_beq;
pub mod from_action_result;
mod is_beta_equal;
mod simplify;
pub mod type_check;

#[derive(Default)]
pub struct TcEnv {
    // uf: UnionFind,
    values: Vec<PartialExpr>,
    tc_id: usize,
    pub errors: Vec<TcError>,

    // Queues
    //TODO readd queued_tc: HashMap<UnionIndex, (Env, UnionIndex)>,
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
