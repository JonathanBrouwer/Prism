use crate::coc::Expr;
use rpds::Vector;
use std::ops::Index;
use crate::coc::type_check::PartialExpr;
use crate::union_find::UnionIndex;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum EnvEntry {
    NType(UnionIndex),
    NSubst(UnionIndex, usize),
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

    // TODO optimization
    // #[must_use]
    // pub fn cons_mut(mut self, e: EnvEntry) -> Self {
    //     self.0.push_back_mut(e);
    //     self
    // }

    // TODO optimization
    pub fn shift(&self, count: usize) -> Self {
        let mut s = self.0.clone();
        assert!(s.len() >= count);
        for _ in 0..count {
            s.drop_last_mut();
        }
        Env(s)
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
