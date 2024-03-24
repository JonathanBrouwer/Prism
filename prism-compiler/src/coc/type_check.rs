use std::marker::PhantomData;
use std::mem;
use crate::coc::env::{Env, EnvEntry};
use crate::coc::env::EnvEntry::*;
use crate::coc::SourceExpr;
use crate::union_find::{UnionFind, UnionIndex};

pub struct TcEnv<'arn> {
    uf: UnionFind,
    uf_values: Vec<PartialExpr<'arn>>,
    errors: Vec<TcError>,
}

#[derive(Clone)]
pub enum PartialExpr<'arn> {
    Type,
    Let(UnionIndex, UnionIndex),
    Var(usize),
    FnType(UnionIndex, UnionIndex),
    FnConstruct(UnionIndex, UnionIndex),
    FnDestruct(UnionIndex, UnionIndex),
    Free,
    Shift(UnionIndex, usize),
    Subst(UnionIndex, (UnionIndex, Env)),
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

    ///Invariant: Returned UnionIndex is valid in Env `s`
    fn tc_expr(&mut self, e: &'arn SourceExpr<'arn>, s: &Env) -> UnionIndex {
        let t = match e {
            SourceExpr::Type => PartialExpr::Type,
            SourceExpr::Let(v, b) => {
                // Check `v`
                let vt = self.tc_expr(v, s);
                self.expect_beq_type(vt, s);

                let vi = self.add_union_index(PartialExpr::SourceExpr((&v, s.clone())));
                let bt =  self.tc_expr(b, &s.cons(CSubst(vi, vt)));
                PartialExpr::Subst(bt, (vi, s.clone()))
            }
            SourceExpr::Var(i) => PartialExpr::Shift(
                match s[*i] {
                    CType(t) => t,
                    CSubst(_, t) => t,
                    _ => unreachable!(),
                },
                i + 1,
            ),
            // SourceExpr::FnType(a, b) => {
            //     let at= self.tc_expr(a, s);
            //     self.expect_beq_type(at, s);
            //     let a = self.add_union_index(PartialExpr::SourceExpr((a, s.clone())));
            //     let bs = s.cons(NType(a));
            //     let bt = self.tc_expr(b, &bs);
            //     self.expect_beq_type(bt, &bs);
            //     PartialExpr::Type
            // }
            // SourceExpr::FnConstruct(a, b) => {
            //     let at = self.tc_expr(a, s);
            //     self.expect_beq_type(at, s);
            //     let a = self.add_union_index(PartialExpr::SourceExpr((a, s.clone())));
            //     let bt = self.tc_expr(b, &s.cons(NType(a)));
            //     PartialExpr::FnType(at, bt)
            // }
            // SourceExpr::FnDestruct(f, a) => {
            //     let ft = self.tc_expr(f, s);
            //     let at = self.tc_expr(a, s);
            //
            //     let rt = self.add_union_index(PartialExpr::Free);
            //     let expect = self.add_union_index(PartialExpr::FnType(at, rt));
            //     self.expect_beq(expect, ft, s);
            //
            //     PartialExpr::Subst(rt, (a, s.clone()))
            // }
            _ => todo!()
        };
        self.add_union_index(t)
    }

    fn add_union_index(&mut self, e: PartialExpr<'arn>) -> UnionIndex {
        self.uf_values.push(e);
        self.uf.add()
    }

    ///Invariant: `a` is valid in `s`
    fn expect_beq_type(&mut self, i: UnionIndex, s: &Env) {
        self.expect_beq(i, Self::type_type(), s)
    }

    ///Invariant: `a` and `b` are valid in `s`
    fn expect_beq(&mut self, i1: UnionIndex, i2: UnionIndex, s: &Env) {
        self.expect_beq_internal(i1, s, i2, s)
    }

    ///Invariant: `a` and `b` are valid in `s`
    fn expect_beq_internal(&mut self, i1: UnionIndex, s1: &Env, i2: UnionIndex, s2: &Env) {
        let (i1, s1) = self.brh(i1, s1.clone());
        let (i2, s2) = self.brh(i2, s2.clone());

        match (&self.uf_values[self.uf.find(i1).0], &self.uf_values[self.uf.find(i2).0]) {
            (&PartialExpr::Type, &PartialExpr::Type) => {
                // If brh returns a Type, we're done. Easy work!
            }
            (&PartialExpr::Var(i1), &PartialExpr::Var(i2)) => {
                // If brh returns a Var, these must be a variable from `sa`/`sb` that is also present in `s`.
                // I don't have a formal proof for this, but I think this is true
                let i1 = i1 - s1.len();
                let i2 = i2 - s2.len();
                if i1 != i2 {
                    self.errors.push(());
                }
            }
            (&PartialExpr::FnType(a1, b1), &PartialExpr::FnType(a2, b2)) => {
                self.expect_beq_internal(a1, &s1, a2, &s2);
                self.expect_beq_internal(b1, &s1.cons(RType), b2, &s2.cons(RType));
            }
            (&PartialExpr::FnConstruct(a1, b1), &PartialExpr::FnConstruct(a2, b2)) => {
                self.expect_beq_internal(a1, &s1, a2, &s2);
                self.expect_beq_internal(b1, &s1.cons(RType), b2, &s2.cons(RType));
            }
            (&PartialExpr::FnDestruct(f1, a1), &PartialExpr::FnDestruct(f2, a2)) => {
                self.expect_beq_internal(f1, &s1, f2, &s2);
                self.expect_beq_internal(a1, &s1, a2, &s2);
            }
            (&PartialExpr::Free, e) | (e, &PartialExpr::Free) => {
                todo!()
            }
            (_e1, _e2) => {
                self.errors.push(());
            }
        }
    }


    fn brh(&mut self, mut start_expr: UnionIndex, mut start_env: Env) -> (UnionIndex, Env) {
        let mut args: Vec<(UnionIndex, Env)> = Vec::new();

        let mut e: UnionIndex = start_expr;
        let mut s: Env = start_env.clone();

        loop {
            // let v = &;
            match self.uf_values[self.uf.find(e).0] {
                PartialExpr::Type => {
                    assert!(args.is_empty());
                    return (e, Env::new())
                }
                PartialExpr::Let(v, b) => {
                    e = b;
                    s = s.cons(RSubst(v, s.clone()))
                }
                PartialExpr::Var(i) => {
                    match s[i] {
                        CType(_) | RType => return if args.len() == 0 {
                            (e, s)
                        } else {
                            (start_expr, start_env)
                        },
                        CSubst(v, _) => {
                            e = v;
                            s = s.shift(i + 1);
                        }
                        RSubst(v, ref vs) => {
                            e = v;
                            s = vs.clone();
                        }
                    }
                }
                PartialExpr::FnType(_, _) => {
                    assert!(args.is_empty());
                    return (e, s)
                }
                PartialExpr::FnConstruct(_, b) => match args.pop() {
                    None => return (e, s),
                    Some((arg, arg_env)) => {
                        e = b;
                        s = s.cons(RSubst(arg, arg_env));

                        // If we're in a state where the stack is empty, we may want to revert to this state later, so save it.
                        if args.is_empty() {
                            start_expr = e;
                            start_env = s.clone();
                        }
                    }
                }
                PartialExpr::FnDestruct(f, a) => {
                    e = f;
                    args.push((a, s.clone()));
                }

                PartialExpr::Shift(b, i) => {
                    e = b;
                    s = s.shift(i);
                }
                PartialExpr::Subst(b, (v, ref vs)) => {
                    e = b;
                    s = s.cons(RSubst(v, vs.clone()))
                }
                PartialExpr::SourceExpr((v, ref vs)) => {
                    match v {
                        SourceExpr::Type => {}
                        SourceExpr::Let(_, _) => {}
                        SourceExpr::Var(_) => {}
                        SourceExpr::FnType(_, _) => {}
                        SourceExpr::FnConstruct(_, _) => {}
                        SourceExpr::FnDestruct(_, _) => {}
                    }
                    todo!()
                }
            }
        }
    }

}