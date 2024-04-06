use std::collections::HashMap;
use crate::coc::env::{Env, UniqueVariableId};
use crate::coc::type_check::TcError;

mod is_beta_equal;
mod beta_reduce;
mod beta_reduce_head;
mod display;
pub mod env;
mod expect_beq;
pub mod from_action_result;
mod simplify;
pub mod type_check;
pub mod exhaustive;

#[derive(Default)]
pub struct TcEnv {
    // uf: UnionFind,
    values: Vec<PartialExpr>,
    tc_id: usize,
    pub errors: Vec<TcError>,
    
    // Queues
    queued_tc: HashMap<UnionIndex, (Env, UnionIndex)>,
    queued_beq: HashMap<UnionIndex, Vec<((Env, HashMap<UniqueVariableId, usize>), (UnionIndex, Env, HashMap<UniqueVariableId, usize>))>>
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct UnionIndex(usize);

#[derive(Copy, Clone)]
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

