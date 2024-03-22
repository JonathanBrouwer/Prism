use crate::coc::SourceExpr;
use rpds::Vector;
use std::ops::Index;
use crate::union_find::UnionIndex;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum EnvEntry {
    // Definitions used during type checking
    CType(UnionIndex),
    CSubst(UnionIndex, UnionIndex),
    
    // Definitions used during beta reduction
    RType,
    RSubst(UnionIndex, Env)
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

    #[must_use]
    pub fn shift(&self, count: usize) -> Self {
        Env(self.0[0..(self.0.len() - count)])
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
