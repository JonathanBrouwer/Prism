use crate::coc::Expr;
use rpds::Vector;
use std::ops::Index;
use crate::coc::type_check::PartialExpr;
use crate::union_find::UnionIndex;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum EnvEntry<'arn> {
    NType(UnionIndex),
    NSubst(UnionIndex, SExpr<'arn>),
}

impl<'arn> EnvEntry<'arn> {
    pub fn typ(&self) -> UnionIndex {
        match self {
            EnvEntry::NType(t) | EnvEntry::NSubst(t, _) => *t
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Env<'arn>(Vector<EnvEntry<'arn>>);

pub type SExpr<'arn> = (&'arn Expr<'arn>, Env<'arn>);

impl<'arn> Env<'arn> {
    pub fn new() -> Self {
        Self(Vector::new())
    }

    #[must_use]
    pub fn cons(&self, e: EnvEntry<'arn>) -> Self {
        Env(self.0.push_back(e))
    }
}

impl<'arn> Index<usize> for Env<'arn> {
    type Output = EnvEntry<'arn>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[self.0.len() - 1 - index]
    }
}
