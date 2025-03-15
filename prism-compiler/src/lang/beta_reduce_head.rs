use crate::lang::CoreIndex;
use crate::lang::env::DbEnv;
use crate::lang::env::EnvEntry::*;
use crate::lang::{CorePrismExpr, PrismEnv};

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn beta_reduce_head(
        &self,
        mut start_expr: CoreIndex,
        mut start_env: DbEnv<'arn>,
    ) -> (CoreIndex, DbEnv<'arn>) {
        let mut args: Vec<(CoreIndex, DbEnv)> = Vec::new();

        let mut e: CoreIndex = start_expr;
        let mut s: DbEnv = start_env;

        loop {
            match self.checked_values[*e] {
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
                    s = s.cons(RSubst(v, s), self.allocs)
                }
                CorePrismExpr::DeBruijnIndex(i) => match s[i] {
                    CType(_, _) | RType(_) => {
                        return if args.is_empty() {
                            (e, s)
                        } else {
                            (start_expr, start_env)
                        };
                    }
                    CSubst(v, _) => {
                        e = v;
                        s = s.shift(i + 1);
                    }
                    RSubst(v, ref vs) => {
                        e = v;
                        s = *vs;
                    }
                },
                CorePrismExpr::FnConstruct(b) => match args.pop() {
                    None => return (e, s),
                    Some((arg, arg_env)) => {
                        e = b;
                        s = s.cons(RSubst(arg, arg_env), self.allocs);
                    }
                },
                CorePrismExpr::FnDestruct(f, a) => {
                    // If we're in a state where the stack is empty, we may want to revert to this state later, so save it.
                    if args.is_empty() {
                        start_expr = e;
                        start_env = s;
                    }
                    // Push new argument to stack
                    e = f;
                    args.push((a, s));
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
