use crate::lang::env::{Env, EnvEntry, UniqueVariableId};
use crate::lang::UnionIndex;
use crate::lang::{PrismEnv, PrismExpr};
use std::collections::HashMap;

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
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

        let e_new = match self.values[*i] {
            PrismExpr::Type => PrismExpr::Type,
            PrismExpr::Let(_, _, _) => unreachable!(),
            PrismExpr::DeBruijnIndex(v) => {
                let EnvEntry::RType(id) = s[v] else {
                    unreachable!()
                };
                PrismExpr::DeBruijnIndex(var_map.len() - var_map[&id] - 1)
            }
            PrismExpr::FnType(n, a, b) => {
                let a = self.beta_reduce_inner(a, &s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.beta_reduce_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PrismExpr::FnType(n, a, b)
            }
            PrismExpr::FnConstruct(n, b) => {
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.beta_reduce_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PrismExpr::FnConstruct(n, b)
            }
            PrismExpr::FnDestruct(a, b) => {
                let a = self.beta_reduce_inner(a, &s, var_map);
                let b = self.beta_reduce_inner(b, &s, var_map);
                PrismExpr::FnDestruct(a, b)
            }
            PrismExpr::Free => PrismExpr::Free,
            PrismExpr::Shift(_, _) => unreachable!(),
            PrismExpr::TypeAssert(_, _) => unreachable!(),
            PrismExpr::Name(..)
            | PrismExpr::ShiftPoint(..)
            | PrismExpr::ShiftTo(..)
            | PrismExpr::ParserValue(..) => {
                unreachable!("Should not occur in typechecked terms")
            }
        };
        self.store(e_new, self.value_origins[*i])
    }
}
