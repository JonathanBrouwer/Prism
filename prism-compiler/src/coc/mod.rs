use crate::union_find::{UnionFind, UnionIndex};

mod beq;
mod br;
mod brh;
mod display;
pub mod env;
mod expect_beq;
pub mod from_action_result;
mod sm;
pub mod type_check;

#[derive(Default)]
pub struct TcEnv {
    uf: UnionFind,
    uf_values: Vec<PartialExpr>,
    errors: Vec<TcError>,
    tc_id: usize,
}

#[derive(Clone)]
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
