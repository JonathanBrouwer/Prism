// use crate::coc::{Expr, SExpr};
// use rpds::Vector;
// use std::ops::Index;
//
// #[derive(Clone, Eq, PartialEq, Debug)]
// pub enum EnvEntry<'a, M: Clone> {
//     NType(&'a Expr<M>),
//     NSubst(SExpr<'a, M>),
// }
//
// #[derive(Clone, Eq, PartialEq, Debug)]
// pub struct Env<'a, M: Clone>(Vector<EnvEntry<'a, M>>);
//
// impl<'a, M: Clone> Env<'a, M> {
//     pub fn new() -> Self {
//         Self(Vector::new())
//     }
//
//     #[must_use]
//     pub fn cons(&self, e: EnvEntry<'a, M>) -> Self {
//         Env(self.0.push_back(e))
//     }
// }
//
// impl<'a, M: Clone> Index<usize> for Env<'a, M> {
//     type Output = EnvEntry<'a, M>;
//
//     fn index(&self, index: usize) -> &Self::Output {
//         &self.0[self.0.len() - 1 - index]
//     }
// }
