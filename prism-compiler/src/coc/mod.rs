use crate::coc::env::{Env, UniqueVariableId};
use std::collections::{HashMap, HashSet};
use crate::coc::error::TcError;

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
mod error;

#[derive(Default)]
pub struct TcEnv {
    // uf: UnionFind,
    values: Vec<PartialExpr>,
    tc_id: usize,
    pub errors: Vec<TcError>,
    toxic_values: HashSet<UnionIndex>,

    // Queues
    queued_beq_free: HashMap<UnionIndex, Vec<((Env, HashMap<UniqueVariableId, usize>), (UnionIndex, Env, HashMap<UniqueVariableId, usize>))>>
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
