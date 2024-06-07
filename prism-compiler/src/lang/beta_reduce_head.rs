use crate::lang::env::Env;
use crate::lang::env::EnvEntry::*;
use crate::lang::{PartialExpr, TcEnv};
use crate::lang::UnionIndex;

impl TcEnv {
    pub fn beta_reduce_head(
        &self,
        mut start_expr: UnionIndex,
        mut start_env: Env,
    ) -> (UnionIndex, Env) {
        let mut args: Vec<(UnionIndex, Env)> = Vec::new();

        let mut e: UnionIndex = start_expr;
        let mut s: Env = start_env.clone();

        loop {
            match self.values[e.0] {
                PartialExpr::Type => {
                    debug_assert!(args.is_empty());
                    return (e, s);
                }
                PartialExpr::Let(v, b) => {
                    e = b;
                    s = s.cons(RSubst(v, s.clone()))
                }
                PartialExpr::DeBruijnIndex(i) => match s[i] {
                    CType(_, _) | RType(_) => {
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
                    debug_assert!(args.is_empty());
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
                        (start_expr, start_env)
                    };
                }
                PartialExpr::Shift(b, i) => {
                    e = b;
                    s = s.shift(i);
                }
            }
        }
    }
}
