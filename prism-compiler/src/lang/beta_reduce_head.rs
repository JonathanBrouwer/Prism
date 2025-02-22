use crate::lang::CheckedIndex;
use crate::lang::env::DbEnv;
use crate::lang::env::EnvEntry::*;
use crate::lang::{CheckedPrismExpr, PrismEnv};

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn beta_reduce_head(
        &self,
        mut start_expr: CheckedIndex,
        mut start_env: DbEnv,
    ) -> (CheckedIndex, DbEnv) {
        let mut args: Vec<(CheckedIndex, DbEnv)> = Vec::new();

        let mut e: CheckedIndex = start_expr;
        let mut s: DbEnv = start_env.clone();

        loop {
            match self.checked_values[*e] {
                // Values
                CheckedPrismExpr::Type
                | CheckedPrismExpr::FnType(_, _)
                | CheckedPrismExpr::GrammarValue(_)
                | CheckedPrismExpr::GrammarType => {
                    assert!(args.is_empty());
                    return (e, s);
                }
                CheckedPrismExpr::Let(v, b) => {
                    e = b;
                    s = s.cons(RSubst(v, s.clone()))
                }
                CheckedPrismExpr::DeBruijnIndex(i) => match s[i] {
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
                        s = vs.clone();
                    }
                },
                CheckedPrismExpr::FnConstruct(b) => match args.pop() {
                    None => return (e, s),
                    Some((arg, arg_env)) => {
                        e = b;
                        s = s.cons(RSubst(arg, arg_env));
                    }
                },
                CheckedPrismExpr::FnDestruct(f, a) => {
                    // If we're in a state where the stack is empty, we may want to revert to this state later, so save it.
                    if args.is_empty() {
                        start_expr = e;
                        start_env = s.clone();
                    }
                    // Push new argument to stack
                    e = f;
                    args.push((a, s.clone()));
                }
                CheckedPrismExpr::Free => {
                    return if args.is_empty() {
                        (e, s)
                    } else {
                        (start_expr, start_env)
                    };
                }
                CheckedPrismExpr::Shift(b, i) => {
                    e = b;
                    s = s.shift(i);
                }
                CheckedPrismExpr::TypeAssert(new_e, _) => {
                    e = new_e;
                }
            }
        }
    }
}
