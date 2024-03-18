use std::marker::PhantomData;
use std::mem;
use crate::coc::env::Env;
use crate::coc::env::EnvEntry::*;
use crate::coc::SourceExpr;
use crate::union_find::{UnionFind, UnionIndex};

pub struct TcEnv<'arn> {
    uf: UnionFind,
    uf_values: Vec<PartialExpr<'arn>>,
    errors: Vec<TcError>,
}

pub enum PartialExpr<'arn> {
    Type,
    Let(UnionIndex, UnionIndex),
    Var(usize),
    FnType(UnionIndex, UnionIndex),
    FnConstruct(UnionIndex, UnionIndex),
    FnDestruct(UnionIndex, UnionIndex),
    Free,
    Shift(UnionIndex, usize),
    Subst(UnionIndex, (&'arn SourceExpr<'arn>, Env)),
    SourceExpr((&'arn SourceExpr<'arn>, Env))
}

pub type TcError = ();

impl<'arn> TcEnv<'arn> {
    pub fn new() -> Self {
        let mut s = Self {
            uf: UnionFind::new(),
            uf_values: Vec::default(),
            errors: Vec::new(),
        };
        let type_type = s.add_union_index(PartialExpr::Type);
        debug_assert_eq!(type_type.0, 0);
        s
    }

    fn type_type() -> UnionIndex {
        UnionIndex(0)
    }

    pub fn type_check(&mut self, expr: &'arn SourceExpr) -> Result<(), Vec<TcError>> {
        self.tc_expr(expr, &Env::new());
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(mem::take(&mut self.errors))
        }
    }

    ///Invariant: UnionIndex is valid in Env `s`
    fn tc_expr(&mut self, e: &'arn SourceExpr<'arn>, s: &Env) -> UnionIndex {
        let t = match e {
            SourceExpr::Type => PartialExpr::Type,
            SourceExpr::Let(v, b) => {
                let vt = self.tc_expr(v, s);
                self.expect_beq_type(vt, s);
                let s= s.cons(NSubst(vt, Self::type_type()));
                let bt =  self.tc_expr(b, &s);
                PartialExpr::Subst(bt, (v, s.clone()))
            }
            SourceExpr::Var(i) => PartialExpr::Shift(
                match s[*i] {
                    NType(t) => t,
                    NSubst(_, t) => t,
                },
                i + 1,
            ),
            SourceExpr::FnType(a, b) => {
                let at= self.tc_expr(a, s);
                self.expect_beq_type(at, s);
                let a = self.add_union_index(PartialExpr::SourceExpr((a, s.clone())));
                let bs = s.cons(NType(a));
                let bt = self.tc_expr(b, &bs);
                self.expect_beq_type(bt, &bs);
                PartialExpr::Type
            }
            SourceExpr::FnConstruct(a, b) => {
                let at = self.tc_expr(a, s);
                self.expect_beq_type(at, s);
                let a = self.add_union_index(PartialExpr::SourceExpr((a, s.clone())));
                let bt = self.tc_expr(b, &s.cons(NType(a)));
                PartialExpr::FnType(at, bt)
            }
            SourceExpr::FnDestruct(f, a) => {
                let ft = self.tc_expr(f, s);
                let at = self.tc_expr(a, s);

                let rt = self.add_union_index(PartialExpr::Free);
                let expect = self.add_union_index(PartialExpr::FnType(at, rt));
                self.expect_beq(expect, ft, s);

                PartialExpr::Subst(rt, (a, s.clone()))
            }
        };
        self.add_union_index(t)
    }

    fn add_union_index(&mut self, e: PartialExpr<'arn>) -> UnionIndex {
        self.uf_values.push(e);
        self.uf.add()
    }

    ///Invariant: `a` is valid in `s`
    fn expect_beq_type(&mut self, a: UnionIndex, s: &Env) {
        self.expect_beq(a, Self::type_type(), s)
    }

    ///Invariant: `a` and `b` are valid in `s`
    fn expect_beq(&mut self, a: UnionIndex, b: UnionIndex, s: &Env) {
        todo!()
    }


}