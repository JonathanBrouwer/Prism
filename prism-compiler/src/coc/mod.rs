use crate::coc::env::Env;
use crate::union_find::{UnionFind, UnionIndex};

mod display;
pub mod env;
pub mod from_action_result;
pub mod type_check;
mod eq;

#[derive(Default)]
pub struct TcEnv {
    uf: UnionFind,
    uf_values: Vec<PartialExpr>,
    errors: Vec<TcError>,
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
    Subst(UnionIndex, (UnionIndex, Env)),
}

pub type TcError = ();
