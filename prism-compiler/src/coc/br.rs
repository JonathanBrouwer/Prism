use crate::coc::env::{Env, EnvEntry, UniqueVariableId};
use crate::coc::{PartialExpr, TcEnv};
use crate::union_find::UnionIndex;
use std::collections::HashMap;

impl TcEnv {
    pub fn br(&mut self, i: UnionIndex) -> UnionIndex {
        self.br_inner(i, &Env::new(), &mut HashMap::new())
    }

    fn br_inner(
        &mut self,
        i: UnionIndex,
        s: &Env,
        var_map: &mut HashMap<UniqueVariableId, usize>,
    ) -> UnionIndex {
        let (i, s) = self.brh(i, s.clone());

        let e_new = match self.uf_values[self.uf.find(i).0] {
            PartialExpr::Type => PartialExpr::Type,
            PartialExpr::Let(_, _) => unreachable!(),
            PartialExpr::Var(v) => {
                let EnvEntry::RType(id) = s[v] else {
                    unreachable!()
                };
                PartialExpr::Var(var_map.len() - var_map[&id] - 1)
            }
            PartialExpr::FnType(a, b) => {
                let a = self.br_inner(a, &s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.br_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PartialExpr::FnType(a, b)
            }
            PartialExpr::FnConstruct(a, b) => {
                let a = self.br_inner(a, &s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.br_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PartialExpr::FnConstruct(a, b)
            }
            PartialExpr::FnDestruct(a, b) => {
                let a = self.br_inner(a, &s, var_map);
                let b = self.br_inner(b, &s, var_map);
                PartialExpr::FnDestruct(a, b)
            }
            PartialExpr::Free => PartialExpr::Free,
            PartialExpr::Shift(_, _) => unreachable!(),
        };
        self.insert_union_index(e_new)
    }
}
