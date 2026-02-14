use crate::lang::CoreIndex;
use crate::lang::env::DbEnv;
use crate::lang::env::EnvEntry::*;
use crate::lang::{CorePrismExpr, PrismDb};

impl PrismDb {
    pub fn beta_reduce_head(
        &self,
        mut start_expr: CoreIndex,
        start_env: &DbEnv,
    ) -> (CoreIndex, DbEnv) {
        let mut args: Vec<(CoreIndex, DbEnv)> = Vec::new();

        let mut e: CoreIndex = start_expr;
        let mut s: DbEnv = start_env.clone();
        let mut start_env = start_env.clone();

        loop {
            match self.values[*e] {
                // Values
                CorePrismExpr::Type
                | CorePrismExpr::FnType(..)
                | CorePrismExpr::GrammarValue(..)
                | CorePrismExpr::GrammarType => {
                    assert!(args.is_empty());
                    return (e, s);
                }
                CorePrismExpr::Let(v, b) => {
                    e = b;
                    let s_clone = s.clone();
                    s = s.cons(RSubst(v, s_clone))
                }
                CorePrismExpr::DeBruijnIndex(i) => match s[i] {
                    CType(_, _) | RType(_) => {
                        return if args.is_empty() {
                            (e, s)
                        } else {
                            (start_expr, start_env.clone())
                        };
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
                CorePrismExpr::FnConstruct(b) => match args.pop() {
                    None => return (e, s),
                    Some((arg, arg_env)) => {
                        e = b;
                        s = s.cons(RSubst(arg, arg_env));
                    }
                },
                CorePrismExpr::FnDestruct(f, a) => {
                    // If we're in a state where the stack is empty, we may want to revert to this state later, so save it.
                    if args.is_empty() {
                        start_expr = e;
                        start_env = s.clone();
                    }
                    // Push new argument to stack
                    e = f;
                    args.push((a, s.clone()));
                }
                CorePrismExpr::Free => {
                    return if args.is_empty() {
                        (e, s)
                    } else {
                        (start_expr, start_env)
                    };
                }
                CorePrismExpr::Shift(b, i) => {
                    e = b;
                    s = s.shift(i);
                }
                CorePrismExpr::TypeAssert(new_e, _) => {
                    e = new_e;
                }
            }
        }
    }
}
