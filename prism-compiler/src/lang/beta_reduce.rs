use crate::lang::env::{Env, EnvEntry, UniqueVariableId};
use crate::lang::{PartialExpr, TcEnv, UnionIndex};
use std::collections::HashMap;

impl TcEnv {
    pub fn beta_reduce(&mut self, i: UnionIndex) -> UnionIndex {
        self.beta_reduce_inner(i, &Env::new(), &mut HashMap::new())
    }

    fn beta_reduce_inner(
        &mut self,
        i: UnionIndex,
        s: &Env,
        var_map: &mut HashMap<UniqueVariableId, usize>,
    ) -> UnionIndex {
        let (i, s) = self.beta_reduce_head(i, s.clone());

        let e_new = match self.values[i.0] {
            PartialExpr::Type => PartialExpr::Type,
            PartialExpr::Let(_, _) => unreachable!(),
            PartialExpr::DeBruijnIndex(v) => {
                let EnvEntry::RType(id) = s[v] else {
                    unreachable!()
                };
                PartialExpr::DeBruijnIndex(var_map.len() - var_map[&id] - 1)
            }
            PartialExpr::FnType(a, b) => {
                let a = self.beta_reduce_inner(a, &s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.beta_reduce_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PartialExpr::FnType(a, b)
            }
            PartialExpr::FnConstruct(a, b) => {
                let a = self.beta_reduce_inner(a, &s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.beta_reduce_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PartialExpr::FnConstruct(a, b)
            }
            PartialExpr::FnDestruct(a, b) => {
                let a = self.beta_reduce_inner(a, &s, var_map);
                let b = self.beta_reduce_inner(b, &s, var_map);
                PartialExpr::FnDestruct(a, b)
            }
            PartialExpr::Free => PartialExpr::Free,
            PartialExpr::Shift(_, _) => unreachable!(),
        };
        self.store(e_new, self.value_origins[i.0])
    }
}
