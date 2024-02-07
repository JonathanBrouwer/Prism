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
    Shift(UnionIndex, usize),
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

    fn brh<'a>(start_expr: &'a PartialExpr<'arn>, start_env: Env, uf_values: &'a [PartialExpr<'arn>]) -> (&'a PartialExpr<'arn>, Env) {
        let mut args = Vec::new();

        let mut e: &'a PartialExpr<'arn> = start_expr;
        let mut s: Env = start_env.clone();

        loop {
            match e {
                PartialExpr::Type => {
                    assert!(args.is_empty());
                    return (&PartialExpr::Type, Env::new())
                }
                PartialExpr::Let(v, b) => {
                    e = &uf_values[b.0];
                    s = s.cons(NSubst(*v, 0))
                }
                PartialExpr::Var(i) => match s[*i] {
                    NType(_) => return if args.len() == 0 {
                        (e, s)
                    } else {
                        (start_expr, start_env)
                    },
                    NSubst(v, shift) => {
                        e = &uf_values[v.0];
                        s.shift(*i + shift + 1);
                    }
                }
                PartialExpr::FnType(_, _) => {
                    assert!(args.is_empty());
                    return (e, s)
                }
                PartialExpr::FnConstruct(_, b) => match args.pop() {
                    None => return (e, s),
                    Some((arg, arg_shift)) => {
                        e = &uf_values[b.0];
                        s = s.cons(NSubst(arg, arg_shift))
                    }
                }
                PartialExpr::FnDestruct(f, a) => {
                    e = &uf_values[f.0];
                    args.push((*a, s.len()));
                }
                PartialExpr::Shift(v, shift) => {
                    e = &uf_values[v.0];
                    s = s.shift(*shift);
                }
                PartialExpr::Free => return (&PartialExpr::Free, Env::new()),
                PartialExpr::Expr(_) => {}
                PartialExpr::Subst(v, i) => {}
            }
        }
    }

    fn expect_beq(&mut self, (ai, ae): (UnionIndex, Env), (bi, be): (UnionIndex, Env)) {
        let (ape, anv) = Self::brh(&self.uf_values[self.uf.find(ai).0], ae.clone(), &self.uf_values).clone();
        let (bpe, bnv) = Self::brh(&self.uf_values[self.uf.find(bi).0], be.clone(), &self.uf_values).clone();

        match (ape, bpe) {
            (PartialExpr::Type, PartialExpr::Type) => {},
            (&PartialExpr::FnType(a1, a2), &PartialExpr::FnType(b1, b2)) => {
                self.expect_beq((a1, anv.clone()), (b1, bnv.clone()));
                self.expect_beq(
                    (a2, anv.cons(NSubst(a1, 0))),
                    (b2, anv.cons(NSubst(b1, 0))),
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
                let s= s.cons(NSubst(vt, 0));
                let bt = self.tc_expr(b, &s);
                PartialExpr::Subst(bt, (v, s.clone()))
            }
            Expr::Var(i) => PartialExpr::Shift(
                match s[*i] {
                    NType(t) => t,
                    NSubst(v, _) => self.get_type(v),
                },
                i + 1,
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