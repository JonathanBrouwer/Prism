use crate::lang::CoreIndex;
use crate::lang::env::DbEnv;
use crate::lang::env::EnvEntry::*;
use crate::lang::{Expr, PrismDb};

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
            match self.exprs[*e] {
                // Values
                Expr::Type | Expr::FnType { .. } => {
                    assert!(args.is_empty());
                    return (e, s);
                }
                Expr::Let {
                    name: _,
                    value: v,
                    body: b,
                } => {
                    e = b;
                    let s_clone = s.clone();
                    s = s.cons(RSubst(v, s_clone))
                }
                Expr::DeBruijnIndex { idx: i } => match s[i] {
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
                Expr::FnConstruct {
                    arg_name: _,
                    arg_type: _,
                    body: b,
                } => match args.pop() {
                    None => return (e, s),
                    Some((arg, arg_env)) => {
                        e = b;
                        s = s.cons(RSubst(arg, arg_env));
                    }
                },
                Expr::FnDestruct {
                    function: f,
                    arg: a,
                } => {
                    // If we're in a state where the stack is empty, we may want to revert to this state later, so save it.
                    if args.is_empty() {
                        start_expr = e;
                        start_env = s.clone();
                    }
                    // Push new argument to stack
                    e = f;
                    args.push((a, s.clone()));
                }
                Expr::Free => {
                    return if args.is_empty() {
                        (e, s)
                    } else {
                        (start_expr, start_env)
                    };
                }
                Expr::Shift(b, i) => {
                    e = b;
                    s = s.shift(i);
                }
                Expr::TypeAssert {
                    value: new_e,
                    type_hint: _,
                } => {
                    e = new_e;
                }
            }
        }
    }
}
