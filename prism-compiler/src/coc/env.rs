use crate::coc::Expr;
use rpds::Vector;
use std::ops::Index;
use crate::coc::type_check::PartialExpr;
use crate::union_find::UnionIndex;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum EnvEntry {
    NType(UnionIndex),
    NSubst(UnionIndex), //TODO don't add type info here because annoying for beta reduce
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Env(Vector<EnvEntry>);

pub type SExpr<'arn> = (&'arn Expr<'arn>, Env);

impl<'arn> Env {
    pub fn new() -> Self {
        Self(Vector::new())
    }

    #[must_use]
    pub fn cons(&self, e: EnvEntry) -> Self {
        Env(self.0.push_back(e))
    }
}

impl<'arn> Index<usize> for Env {
    type Output = EnvEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[self.0.len() - 1 - index]
    }
}
