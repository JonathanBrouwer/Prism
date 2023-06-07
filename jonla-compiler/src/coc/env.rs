use crate::coc::{Expr, SExpr};
use rpds::Vector;
use std::ops::Index;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum EnvEntry<'a> {
    NType(&'a Expr),
    NSubst(SExpr<'a>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Env<'a>(Vector<EnvEntry<'a>>);

impl<'a> Env<'a> {
    pub fn new() -> Self {
        Self(Vector::new())
    }

    #[must_use]
    pub fn cons(&self, e: EnvEntry<'a>) -> Self {
        Env(self.0.push_back(e))
    }
}

impl<'a> Index<usize> for Env<'a> {
    type Output = EnvEntry<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[self.0.len() - 1 - index]
    }
}
