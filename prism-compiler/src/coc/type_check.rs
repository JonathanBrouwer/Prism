use std::mem;

use crate::coc::{PartialExpr, TcEnv, TcError};
use crate::coc::env::Env;
use crate::coc::env::EnvEntry::*;
use crate::union_find::UnionIndex;

impl TcEnv {
    fn type_type() -> UnionIndex {
        UnionIndex(0)
    }

    pub fn type_check(&mut self, root: UnionIndex) -> Result<UnionIndex, Vec<TcError>> {
        let ti = self.tc_expr(root, &Env::new());
        if self.errors.is_empty() {
            Ok(ti)
        } else {
            Err(mem::take(&mut self.errors))
        }
    }

    ///Invariant: Returned UnionIndex is valid in Env `s`
    fn tc_expr(&mut self, i: UnionIndex, s: &Env) -> UnionIndex {
        let t = match self.uf_values[self.uf.find(i).0] {
            PartialExpr::Type => PartialExpr::Type,
            PartialExpr::Let(v, b) => {
                // Check `v`
                let vt = self.tc_expr(v, s);
                self.expect_beq_type(vt, s);

                let bt = self.tc_expr(b, &s.cons(CSubst(v, vt)));
                PartialExpr::Subst(bt, (v, s.clone()))
            }
            PartialExpr::Var(i) => PartialExpr::Shift(
                match s[i] {
                    CType(t) => t,
                    CSubst(_, t) => t,
                    _ => unreachable!(),
                },
                i + 1,
            ),
            PartialExpr::FnType(a, b) => {
                let at = self.tc_expr(a, s);
                self.expect_beq_type(at, s);
                let bs = s.cons(CType(a));
                let bt = self.tc_expr(b, &bs);
                self.expect_beq_type(bt, &bs);
                PartialExpr::Type
            }
            PartialExpr::FnConstruct(a, b) => {
                let at = self.tc_expr(a, s);
                self.expect_beq_type(at, s);
                let bt = self.tc_expr(b, &s.cons(CType(a)));
                PartialExpr::FnType(at, bt)
            }
            PartialExpr::FnDestruct(f, a) => {
                let ft = self.tc_expr(f, s);
                let at = self.tc_expr(a, s);

                let rt = self.insert_union_index(PartialExpr::Free);
                let expect = self.insert_union_index(PartialExpr::FnType(at, rt));
                self.expect_beq(expect, ft, s);

                PartialExpr::Subst(rt, (a, s.clone()))
            }
            PartialExpr::Free | PartialExpr::Shift(..) | PartialExpr::Subst(..) => unreachable!(),
        };
        self.insert_union_index(t)
    }

    pub fn insert_union_index(&mut self, e: PartialExpr) -> UnionIndex {
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
        // Brh and reduce i1 and i2
        let (i1, s1) = self.brh(i1, s1.clone());
        let (i2, s2) = self.brh(i2, s2.clone());
        let i1 = self.uf.find(i1);
        let i2 = self.uf.find(i2);

        match (&self.uf_values[i1.0], &self.uf_values[i2.0]) {
            (&PartialExpr::Type, &PartialExpr::Type) => {
                // If beta_reduce returns a Type, we're done. Easy work!
            }
            (&PartialExpr::Var(i1), &PartialExpr::Var(i2)) => {
                // If beta_reduce returns a Var, these must be a variable from `sa`/`sb` that is also present in `s`.
                // I don't have a formal proof for this, but I think this is true
                // We want i1 - s1.len() == i2 - s2.len()
                // This is equivalent to i1 + s2.len() == i2 + s1.len(), and avoids overflow issues
                let i1 = i1 + s2.len();
                let i2 = i2 + s1.len();
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
            (_e, &PartialExpr::Free) => {
                self.uf.union_left(i1, i2);
            }
            (&PartialExpr::Free, _e) => {
                self.uf.union_left(i2, i1);
            }
            (_e1, _e2) => {
                self.errors.push(());
            }
        }
    }

    pub fn brh(&mut self, mut start_expr: UnionIndex, mut start_env: Env) -> (UnionIndex, Env) {
        let mut args: Vec<(UnionIndex, Env)> = Vec::new();

        let mut e: UnionIndex = start_expr;
        let mut s: Env = start_env.clone();

        loop {
            match self.uf_values[self.uf.find(e).0] {
                PartialExpr::Type => {
                    assert!(args.is_empty());
                    return (e, s);
                }
                PartialExpr::Let(v, b) => {
                    e = b;
                    s = s.cons(RSubst(v, s.clone()))
                }
                PartialExpr::Var(i) => match s[i] {
                    CType(_) | RType => {
                        return if args.is_empty() {
                            (e, s)
                        } else {
                            (start_expr, start_env)
                        }
                    }
                    CSubst(v, _) => {
                        e = v;
                        s = s.shift(i + 1);
                    }
                    RSubst(v, ref vs) => {
                        e = v;
                        s = vs.clone();
                    }
                },
                PartialExpr::FnType(_, _) => {
                    assert!(args.is_empty());
                    return (e, s);
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
                },
                PartialExpr::FnDestruct(f, a) => {
                    e = f;
                    args.push((a, s.clone()));
                }
                PartialExpr::Free => {
                    return if args.is_empty() {
                        (e, s)
                    } else {
                        //TODO is this correct?
                        (start_expr, start_env)
                    };
                }
                PartialExpr::Shift(b, i) => {
                    e = b;
                    s = s.shift(i);
                }
                PartialExpr::Subst(b, (v, ref vs)) => {
                    e = b;
                    s = s.cons(RSubst(v, vs.clone()))
                }
            }
        }
    }
}
