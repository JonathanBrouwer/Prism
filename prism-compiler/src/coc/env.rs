use crate::coc::SourceExpr;
use rpds::Vector;
use std::ops::Index;
use crate::union_find::UnionIndex;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum EnvEntry {
    NType(UnionIndex),
    NSubst(UnionIndex, UnionIndex),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Env(Vector<EnvEntry>);

pub type SExpr<'arn> = (&'arn SourceExpr<'arn>, Env);

impl<'arn> Env {
    pub fn new() -> Self {
        Self(Vector::new())
    }

    #[must_use]
    pub fn cons(&self, e: EnvEntry) -> Self {
        Env(self.0.push_back(e))
    }
    
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'arn> Index<usize> for Env {
    type Output = EnvEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[self.0.len() - 1 - index]
    }
}
