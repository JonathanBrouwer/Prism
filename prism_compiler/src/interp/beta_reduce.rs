use crate::lang::Expr;
use crate::lang::env::{DbEnv, EnvEntry};
use crate::lang::{CoreIndex, PrismDb};
use crate::type_check::{TypecheckPrismEnv, UniqueVariableId};
use std::collections::HashMap;

impl PrismDb {
    pub fn beta_reduce(&mut self, i: CoreIndex, env: &DbEnv) -> CoreIndex {
        let mut tc_env = TypecheckPrismEnv::new(self);
        tc_env.beta_reduce_inner(i, env, &mut HashMap::new())
    }
}

impl<'a> TypecheckPrismEnv<'a> {
    fn beta_reduce_inner(
        &mut self,
        i: CoreIndex,
        s: &DbEnv,
        var_map: &mut HashMap<UniqueVariableId, usize>,
    ) -> CoreIndex {
        let (i, s) = self.db.beta_reduce_head(i, s);

        let e_new = match self.db.exprs[*i] {
            // Values
            Expr::Type => {
                return i;
            }

            Expr::Let {
                name: _,
                value: _,
                body: _,
            } => unreachable!(),
            Expr::DeBruijnIndex { idx: v } => {
                let EnvEntry::RType(id) = s[v] else {
                    unreachable!()
                };
                Expr::DeBruijnIndex {
                    idx: var_map.len() - var_map[&id] - 1,
                }
            }
            Expr::FnType {
                arg_name,
                arg_type: a,
                body: b,
            } => {
                let a = self.beta_reduce_inner(a, &s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let sub_env = s.cons(EnvEntry::RType(id));
                let b = self.beta_reduce_inner(b, &sub_env, var_map);
                var_map.remove(&id);
                Expr::FnType {
                    arg_name,
                    arg_type: a,
                    body: b,
                }
            }
            Expr::FnConstruct {
                arg_name,
                arg_type,
                body: b,
            } => {
                let arg_type = self.beta_reduce_inner(arg_type, &s, var_map);

                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let sub_env = s.cons(EnvEntry::RType(id));
                let b = self.beta_reduce_inner(b, &sub_env, var_map);
                var_map.remove(&id);
                Expr::FnConstruct {
                    arg_name,
                    arg_type,
                    body: b,
                }
            }
            Expr::FnDestruct {
                function: a,
                arg: b,
            } => {
                let a = self.beta_reduce_inner(a, &s, var_map);
                let b = self.beta_reduce_inner(b, &s, var_map);
                Expr::FnDestruct {
                    function: a,
                    arg: b,
                }
            }
            Expr::Free => Expr::Free,
            Expr::Shift(_, _) => unreachable!(),
            Expr::TypeAssert {
                value: _,
                type_hint: _,
            } => unreachable!(),
        };
        self.db.store(e_new, self.db.expr_origins[*i])
    }
}
