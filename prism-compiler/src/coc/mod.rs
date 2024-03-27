use crate::union_find::{UnionFind, UnionIndex};

mod is_beta_equal;
mod beta_reduce;
mod beta_reduce_head;
mod display;
pub mod env;
mod expect_beq;
pub mod from_action_result;
mod simplify;
pub mod type_check;

#[derive(Default)]
pub struct TcEnv {
    uf: UnionFind,
    uf_values: Vec<PartialExpr>,
    errors: Vec<TcError>,
    tc_id: usize,
}

impl TcEnv {
    pub fn errors(&self) -> &[TcError] {
        &self.errors
    }
}

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

pub type TcError = ();
