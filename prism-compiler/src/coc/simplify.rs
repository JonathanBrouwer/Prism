use crate::coc::env::{Env, EnvEntry, UniqueVariableId};
use crate::coc::UnionIndex;
use crate::coc::{PartialExpr, TcEnv};
use std::collections::HashMap;

impl TcEnv {
    pub fn simplify(&mut self, i: UnionIndex) -> UnionIndex {
        self.simplify_inner(i, &Env::new(), &mut HashMap::new())
    }

    fn simplify_inner(
        &mut self,
        i: UnionIndex,
        s: &Env,
        var_map: &mut HashMap<UniqueVariableId, usize>,
    ) -> UnionIndex {
        let e_new = match self.values[i.0] {
            PartialExpr::Type => PartialExpr::Type,
            PartialExpr::Let(v, b) => {
                let v = self.simplify_inner(v, s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PartialExpr::Let(v, b)
            }
            PartialExpr::Var(v) => match &s[v] {
                EnvEntry::CType(_, _) | EnvEntry::CSubst(_, _) => unreachable!(),
                EnvEntry::RType(id) => PartialExpr::Var(var_map.len() - var_map[id] - 1),
                EnvEntry::RSubst(subst, subst_env) => {
                    return self.simplify_inner(*subst, subst_env, var_map)
                }
            },
            PartialExpr::FnType(a, b) => {
                let a = self.simplify_inner(a, s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PartialExpr::FnType(a, b)
            }
            PartialExpr::FnConstruct(a, b) => {
                let a = self.simplify_inner(a, s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PartialExpr::FnConstruct(a, b)
            }
            PartialExpr::FnDestruct(a, b) => {
                let a = self.simplify_inner(a, s, var_map);
                let b = self.simplify_inner(b, s, var_map);
                PartialExpr::FnDestruct(a, b)
            }
            PartialExpr::Free => PartialExpr::Free,
            PartialExpr::Shift(b, i) => return self.simplify_inner(b, &s.shift(i), var_map),
        };
        self.store(e_new)
    }
}
