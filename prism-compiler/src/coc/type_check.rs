use prism_parser::parser::parser_instance::Arena;
use crate::coc::env::{Env, SExpr};
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
    pub types: Vec<PartialExpr<'arn>>,
}

impl<'arn> TcEnv<'arn> {
    pub fn new() -> Self {
        Self {
            uf: UnionFind::new(),
            types: Vec::new(),
        }
    }

    pub fn brh<'a>(expr: &'a PartialExpr, env: Env<'arn>, types: &'a Vec<PartialExpr<'arn>>) -> &'a PartialExpr<'arn> {


        todo!()
    }

    pub fn expect_beq(&mut self, (ai, ae): (UnionIndex, Env<'arn>), (bi, be): (UnionIndex, Env<'arn>)) {
        let ai = self.uf.find(ai);
        let bi = self.uf.find(bi);

        let ape = Self::brh(&self.types[ai.0], ae.clone(), &self.types).clone();
        let bpe = Self::brh(&self.types[bi.0], be.clone(), &self.types).clone();

        match (ape, bpe) {
            (PartialExpr::Type, PartialExpr::Type) => {},
            (PartialExpr::FnType(a1, a2), PartialExpr::FnType(b1, b2)) => {
                self.expect_beq((a1, ae.clone()), (b1, be.clone()));
                self.expect_beq((a2, ae), (a2, be));
            }
            (_, _) => {}
        }




        // todo!()
    }

    pub fn expect_beq_type(&mut self, a: (UnionIndex, Env<'arn>)) {
        let typ = self.add_union_index(PartialExpr::Type);
        self.expect_beq(a, (typ, Env::new()))
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
                let s= s.cons(NSubst(vt, (v, s.clone())));
                let bt = self.tc_expr(b, &s);
                PartialExpr::Subst(bt, (v, s.clone()))
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