use crate::lang::CorePrismExpr;
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

        let e_new = match self.db.checked_values[*i] {
            // Values
            CorePrismExpr::Type | CorePrismExpr::GrammarValue(..) | CorePrismExpr::GrammarType => {
                return i;
            }

            CorePrismExpr::Let(_, _) => unreachable!(),
            CorePrismExpr::DeBruijnIndex(v) => {
                let EnvEntry::RType(id) = s[v] else {
                    unreachable!()
                };
                CorePrismExpr::DeBruijnIndex(var_map.len() - var_map[&id] - 1)
            }
            CorePrismExpr::FnType(a, b) => {
                let a = self.beta_reduce_inner(a, &s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let sub_env = s.cons(EnvEntry::RType(id));
                let b = self.beta_reduce_inner(b, &sub_env, var_map);
                var_map.remove(&id);
                CorePrismExpr::FnType(a, b)
            }
            CorePrismExpr::FnConstruct(b) => {
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let sub_env = s.cons(EnvEntry::RType(id));
                let b = self.beta_reduce_inner(b, &sub_env, var_map);
                var_map.remove(&id);
                CorePrismExpr::FnConstruct(b)
            }
            CorePrismExpr::FnDestruct(a, b) => {
                let a = self.beta_reduce_inner(a, &s, var_map);
                let b = self.beta_reduce_inner(b, &s, var_map);
                CorePrismExpr::FnDestruct(a, b)
            }
            CorePrismExpr::Free => CorePrismExpr::Free,
            CorePrismExpr::Shift(_, _) => unreachable!(),
            CorePrismExpr::TypeAssert(_, _) => unreachable!(),
        };
        self.db.store_checked(e_new, self.db.checked_origins[*i])
    }
}
