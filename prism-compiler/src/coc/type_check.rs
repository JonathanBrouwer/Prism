use prism_parser::parser::parser_instance::Arena;
use crate::coc::env::{Env, SExpr};
use crate::coc::env::EnvEntry::{NSubst, NType};
use crate::coc::{beta, Expr};
use crate::union_find::{UnionFind, UnionIndex};

pub fn tc_root<'arn>(e: &'arn Expr<'arn>, arena: &'arn Arena<Expr<'arn>>) -> Result<(), ()> {
    let mut env = TcEnv::new(arena);
    env.tc_expr(e, &Env::new());
    Ok(())
}

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
    pub arena: &'arn Arena<Expr<'arn>>,
    pub uf: UnionFind,
    pub types: Vec<PartialExpr<'arn>>,
}

impl<'arn> TcEnv<'arn> {
    pub fn new(arena: &'arn Arena<Expr<'arn>>) -> Self {
        Self {
            arena,
            uf: UnionFind::new(),
            types: Vec::new(),
        }
    }

    pub fn expect_beq(&mut self, a: (UnionIndex, Env<'arn>), b: (UnionIndex, Env<'arn>)) {
        todo!()
    }

    pub fn expect_beq_type(&mut self, a: (UnionIndex, Env<'arn>)) {
        todo!()
    }

    fn add_union_index(&mut self, e: PartialExpr<'arn>) -> UnionIndex {
        self.types.push(e);
        self.uf.add()
    }

    pub fn tc_expr(&mut self, e: &'arn Expr<'arn>, s: &Env<'arn>) -> UnionIndex {
        let t = match e {
            Expr::Type => PartialExpr::Type,
            Expr::Let(v, b) => {
                let vt = self.tc_expr(v, s);
                self.expect_beq_type((vt, s.clone()));
                let bt = self.tc_expr(b, &s.cons(NSubst(vt, (v, s.clone()))));
                PartialExpr::Shift(bt, -1)
            }
            Expr::Var(i) => PartialExpr::Shift(
                s[*i].typ(),
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