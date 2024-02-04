use std::marker::PhantomData;
use std::mem;
use prism_parser::parser::parser_instance::Arena;
use crate::coc::env::{Env, EnvEntry, SExpr};
use crate::coc::env::EnvEntry::{NSubst, NType};
use crate::coc::{brh_expr, Expr};
use crate::union_find::{UnionFind, UnionIndex};

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PartialExpr<'arn> {
    Type,
    Let(UnionIndex, UnionIndex),
    Var(usize),
    FnType(UnionIndex, UnionIndex),
    FnConstruct(UnionIndex, UnionIndex),
    FnDestruct(UnionIndex, UnionIndex),
    Shift(UnionIndex, isize),
    Free,
    Expr(&'arn Expr<'arn>),
    Subst(UnionIndex, SExpr<'arn>),
}

pub struct TcEnv<'arn> {
    pub uf: UnionFind,
    pub types: PhantomData<PartialExpr<'arn>>,
    pub errors: Vec<TcError>,
}

pub type TcError = ();

impl<'arn> TcEnv<'arn> {
    pub fn new() -> Self {
        Self {
            uf: UnionFind::new(),
            types: PhantomData::default(),
            errors: Vec::new(),
        }
    }

    pub fn type_check(&mut self, expr: &'arn Expr<'arn>) -> Result<(), Vec<TcError>> {
        self.tc_expr(expr, &Env::new());
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(mem::take(&mut self.errors))
        }

    }

    fn brh<'a>(expr: &'a PartialExpr<'arn>, env: Env) -> (&'a PartialExpr<'arn>, Env) {

        (expr, env) //TODO
    }

    fn expect_beq(&mut self, (ai, ae): (UnionIndex, Env), (bi, be): (UnionIndex, Env)) {
        let ai = self.uf.find(ai);
        let bi = self.uf.find(bi);

        //todo don't use self.types
        let (ape, anv) = Self::brh(&self.types[ai.0], ae.clone(), &self.types).clone();
        let (bpe, bnv) = Self::brh(&self.types[bi.0], be.clone(), &self.types).clone();

        match (ape, bpe) {
            (PartialExpr::Type, PartialExpr::Type) => {},
            (&PartialExpr::FnType(a1, a2), &PartialExpr::FnType(b1, b2)) => {
                self.expect_beq((a1, anv.clone()), (b1, bnv.clone()));
                self.expect_beq(
                    (a2, anv.cons(NSubst(a1))),
                    (b2, anv.cons(NSubst(b1))),
                )
                // self.expect_beq((a2, anv), (a2, bnv));
            }
            (PartialExpr::FnConstruct(a1, a2), PartialExpr::FnConstruct(b1, b2)) => {
                self.expect_beq((*a1, anv.clone()), (*b1, bnv.clone()));
                // TODO

            }
            _ => {
                self.errors.push(());
            }
        }
    }

    fn expect_beq_type(&mut self, a: (UnionIndex, Env)) {
        let typ = self.add_union_index(PartialExpr::Type);
        self.expect_beq(a, (typ, Env::new()))
    }

    fn add_union_index(&mut self, e: PartialExpr<'arn>) -> UnionIndex {
        self.types.push(e);
        self.uf.add()
    }

    fn tc_expr(&mut self, e: &'arn Expr<'arn>, s: &Env) -> UnionIndex {
        let t = match e {
            Expr::Type => PartialExpr::Type,
            Expr::Let(v, b) => {
                let vt = self.tc_expr(v, s);
                self.expect_beq_type((vt, s.clone()));
                let s= s.cons(NSubst(vt));
                let bt = self.tc_expr(b, &s);
                PartialExpr::Subst(bt, (v, s.clone()))
            }
            Expr::Var(i) => PartialExpr::Shift(
                match s[*i] {
                    NType(t) => t,
                    NSubst(v) => self.types[v.0],
                },
                (i + 1) as isize,
            ),
            Expr::FnType(a, b) => {
                let at= self.tc_expr(a, s);
                self.expect_beq_type((at, s.clone()));
                let a = self.add_union_index(PartialExpr::Expr(a));
                let bt = self.tc_expr(b, &s.cons(NType(a)));
                self.expect_beq_type((bt, s.clone()));
                PartialExpr::Type
            }
            Expr::FnConstruct(a, b) => {
                let at = self.tc_expr(a, s);
                self.expect_beq_type((at, s.clone()));
                let a = self.add_union_index(PartialExpr::Expr(a));
                let bt = self.tc_expr(b, &s.cons(NType(a)));
                PartialExpr::FnType(at, bt)
            }
            Expr::FnDestruct(f, a) => {
                let ft = self.tc_expr(f, s);
                let at = self.tc_expr(a, s);

                let rt = self.add_union_index(PartialExpr::Free);
                let expect = self.add_union_index(PartialExpr::FnType(at, rt));
                self.expect_beq((expect, s.clone()), (ft, s.clone()));

                PartialExpr::Subst(rt, (a, s.clone()))
            }
        };
        self.add_union_index(t)
    }
}