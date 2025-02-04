use crate::lang::CheckedIndex;
use crate::lang::env::{Env, EnvEntry, UniqueVariableId};
use crate::lang::{CheckedPrismExpr, PrismEnv};
use std::collections::HashMap;

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn beta_reduce(&mut self, i: CheckedIndex) -> CheckedIndex {
        self.beta_reduce_inner(i, &Env::new(), &mut HashMap::new())
    }

    fn beta_reduce_inner(
        &mut self,
        i: CheckedIndex,
        s: &Env,
        var_map: &mut HashMap<UniqueVariableId, usize>,
    ) -> CheckedIndex {
        let (i, s) = self.beta_reduce_head(i, s.clone());

        let e_new = match self.checked_values[*i] {
            // Values
            CheckedPrismExpr::Type
            | CheckedPrismExpr::ParserValue(..)
            | CheckedPrismExpr::ParsedType => self.checked_values[*i],

            CheckedPrismExpr::Let(_, _) => unreachable!(),
            CheckedPrismExpr::DeBruijnIndex(v) => {
                let EnvEntry::RType(id) = s[v] else {
                    unreachable!()
                };
                CheckedPrismExpr::DeBruijnIndex(var_map.len() - var_map[&id] - 1)
            }
            CheckedPrismExpr::FnType(a, b) => {
                let a = self.beta_reduce_inner(a, &s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.beta_reduce_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                CheckedPrismExpr::FnType(a, b)
            }
            CheckedPrismExpr::FnConstruct(b) => {
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.beta_reduce_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                CheckedPrismExpr::FnConstruct(b)
            }
            CheckedPrismExpr::FnDestruct(a, b) => {
                let a = self.beta_reduce_inner(a, &s, var_map);
                let b = self.beta_reduce_inner(b, &s, var_map);
                CheckedPrismExpr::FnDestruct(a, b)
            }
            CheckedPrismExpr::Free => CheckedPrismExpr::Free,
            CheckedPrismExpr::Shift(_, _) => unreachable!(),
            CheckedPrismExpr::TypeAssert(_, _) => unreachable!(),
        };
        self.store_checked(e_new, self.checked_origins[*i])
    }
}
