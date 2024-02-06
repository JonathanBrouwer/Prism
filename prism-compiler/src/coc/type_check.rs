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
    uf: UnionFind,
    uf_values: Vec<PartialExpr<'arn>>,
    uf_types: Vec<UnionIndex>,
    errors: Vec<TcError>,
}

impl<'arn> TcEnv<'arn> {
    fn get_val(&self, v: UnionIndex) -> &PartialExpr<'arn> {
        &self.uf_values[v.0]
    }

    fn get_type(&self, v: UnionIndex) -> UnionIndex {
        self.uf_types[v.0]
    }

    fn type_type() -> UnionIndex {
        UnionIndex(0)
    }
}

pub type TcError = ();

impl<'arn> TcEnv<'arn> {
    pub fn new() -> Self {
        let mut s = Self {
            uf: UnionFind::new(),
            uf_values: Vec::default(),
            uf_types: Vec::default(),
            errors: Vec::new(),
        };
        let type_type = s.add_union_index(PartialExpr::Type, None);
        debug_assert_eq!(type_type.0, 0);
        s
    }

    pub fn type_check(&mut self, expr: &'arn Expr<'arn>) -> Result<(), Vec<TcError>> {
        self.tc_expr(expr, &Env::new());
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(mem::take(&mut self.errors))
        }

    }

    fn brh<'a>(expr: &'a PartialExpr<'arn>, env: Env, uf_values: &'a [PartialExpr<'arn>]) -> (&'a PartialExpr<'arn>, Env) {

        (expr, env) //TODO
    }

    fn expect_beq(&mut self, (ai, ae): (UnionIndex, Env), (bi, be): (UnionIndex, Env)) {
        let ai = self.uf.find(ai);
        let bi = self.uf.find(bi);

        //todo don't use self.types
        let (ape, anv) = Self::brh(&self.uf_values[ai.0], ae.clone(), &self.uf_values).clone();
        let (bpe, bnv) = Self::brh(&self.uf_values[bi.0], be.clone(), &self.uf_values).clone();

        match (ape, bpe) {
            (PartialExpr::Type, PartialExpr::Type) => {},
            (&PartialExpr::FnType(a1, a2), &PartialExpr::FnType(b1, b2)) => {
                self.expect_beq((a1, anv.clone()), (b1, bnv.clone()));
                self.expect_beq(
                    (a2, anv.cons(NSubst(a1))),
                    (b2, anv.cons(NSubst(b1))),
                )
            }
            (PartialExpr::FnConstruct(a1, a2), PartialExpr::FnConstruct(b1, b2)) => {
                self.expect_beq((*a1, anv.clone()), (*b1, bnv.clone()));
                // TODO

            }
            (e1, e2) => {
                println!("Beq failed: {e1:?} / {e2:?}");
                self.errors.push(());
            }
        }
    }

    fn expect_beq_type(&mut self, a: (UnionIndex, Env)) {
        let typ = self.add_union_index(PartialExpr::Type, None);
        self.expect_beq(a, (typ, Env::new()))
    }

    fn add_union_index(&mut self, e: PartialExpr<'arn>, t: Option<UnionIndex>) -> UnionIndex {
        self.uf_values.push(e);
        self.uf_types.push(t.unwrap_or(Self::type_type()));
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
                    NSubst(v) => self.get_type(v),
                },
                (i + 1) as isize,
            ),
            Expr::FnType(a, b) => {
                let at= self.tc_expr(a, s);
                self.expect_beq_type((at, s.clone()));
                let a = self.add_union_index(PartialExpr::Expr(a), Some(at));
                let bt = self.tc_expr(b, &s.cons(NType(a)));
                self.expect_beq_type((bt, s.clone()));
                PartialExpr::Type
            }
            Expr::FnConstruct(a, b) => {
                let at = self.tc_expr(a, s);
                self.expect_beq_type((at, s.clone()));
                let a = self.add_union_index(PartialExpr::Expr(a), Some(at));
                let bt = self.tc_expr(b, &s.cons(NType(a)));
                PartialExpr::FnType(at, bt)
            }
            Expr::FnDestruct(f, a) => {
                let ft = self.tc_expr(f, s);
                let at = self.tc_expr(a, s);

                let rt = self.add_union_index(PartialExpr::Free, None);
                let expect = self.add_union_index(PartialExpr::FnType(at, rt), None);
                self.expect_beq((expect, s.clone()), (ft, s.clone()));

                PartialExpr::Subst(rt, (a, s.clone()))
            }
        };
        self.add_union_index(t, None)
    }
}