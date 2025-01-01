use crate::lang::env::Env;
use crate::lang::env::EnvEntry::*;
use crate::lang::UnionIndex;
use crate::lang::{PartialExpr, TcEnv};

impl<'grm> TcEnv<'grm> {
    pub fn beta_reduce_head(
        &self,
        mut start_expr: UnionIndex,
        mut start_env: Env<'grm>,
    ) -> (UnionIndex, Env<'grm>) {
        let mut args: Vec<(UnionIndex, Env<'grm>)> = Vec::new();

        let mut e: UnionIndex = start_expr;
        let mut s: Env = start_env.clone();

        loop {
            match self.values[*e] {
                PartialExpr::Type => {
                    assert!(args.is_empty());
                    return (e, s);
                }
                PartialExpr::Let(_n, v, b) => {
                    e = b;
                    s = s.cons(RSubst(v, s.clone()))
                }
                PartialExpr::DeBruijnIndex(i) => match s[i] {
                    CType(_, _, _) | RType(_) => {
                        return if args.is_empty() {
                            (e, s)
                        } else {
                            (start_expr, start_env)
                        }
                    }
                    CSubst(v, _, _) => {
                        e = v;
                        s = s.shift(i + 1);
                    }
                    RSubst(v, ref vs) => {
                        e = v;
                        s = vs.clone();
                    }
                },
                PartialExpr::FnType(_n, _, _) => {
                    assert!(args.is_empty());
                    return (e, s);
                }
                PartialExpr::FnConstruct(_n, b) => match args.pop() {
                    None => return (e, s),
                    Some((arg, arg_env)) => {
                        e = b;
                        s = s.cons(RSubst(arg, arg_env));
                    }
                },
                PartialExpr::FnDestruct(f, a) => {
                    // If we're in a state where the stack is empty, we may want to revert to this state later, so save it.
                    if args.is_empty() {
                        start_expr = e;
                        start_env = s.clone();
                    }
                    // Push new argument to stack
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
                PartialExpr::TypeAssert(new_e, _) => {
                    e = new_e;
                }
                PartialExpr::Name(_) => unreachable!("Should not occur in typechecked terms"),
            }
        }
    }
}
