use crate::coc::env::Env;
use crate::union_find::{UnionFind, UnionIndex};

mod display;
pub mod env;
pub mod from_action_result;
pub mod type_check;

pub struct TcEnv {
    uf: UnionFind,
    uf_values: Vec<PartialExpr>,
    errors: Vec<TcError>,
    pub root: UnionIndex,
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
