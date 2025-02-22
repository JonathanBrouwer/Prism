use crate::lang::CheckedIndex;
use crate::lang::env::{DbEnv, EnvEntry, UniqueVariableId};
use crate::lang::{CheckedPrismExpr, PrismEnv};
use std::collections::HashMap;

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn simplify(&mut self, i: CheckedIndex) -> CheckedIndex {
        self.simplify_inner(i, &DbEnv::new(), &mut HashMap::new())
    }

    fn simplify_inner(
        &mut self,
        i: CheckedIndex,
        s: &DbEnv,
        var_map: &mut HashMap<UniqueVariableId, usize>,
    ) -> CheckedIndex {
        let e_new = match self.checked_values[*i] {
            CheckedPrismExpr::Type => CheckedPrismExpr::Type,
            CheckedPrismExpr::Let(v, b) => {
                let v = self.simplify_inner(v, s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                CheckedPrismExpr::Let(v, b)
            }
            CheckedPrismExpr::DeBruijnIndex(v) => match s.get(v) {
                Some(EnvEntry::CType(_, _)) | Some(EnvEntry::CSubst(_, _)) => unreachable!(),
                Some(EnvEntry::RType(id)) => {
                    CheckedPrismExpr::DeBruijnIndex(var_map.len() - var_map[id] - 1)
                }
                Some(EnvEntry::RSubst(subst, subst_env)) => {
                    return self.simplify_inner(*subst, subst_env, var_map);
                }
                None => CheckedPrismExpr::DeBruijnIndex(v),
            },
            CheckedPrismExpr::FnType(a, b) => {
                let a = self.simplify_inner(a, s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                CheckedPrismExpr::FnType(a, b)
            }
            CheckedPrismExpr::FnConstruct(b) => {
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                CheckedPrismExpr::FnConstruct(b)
            }
            CheckedPrismExpr::FnDestruct(a, b) => {
                let a = self.simplify_inner(a, s, var_map);
                let b = self.simplify_inner(b, s, var_map);
                CheckedPrismExpr::FnDestruct(a, b)
            }
            CheckedPrismExpr::Free => CheckedPrismExpr::Free,
            CheckedPrismExpr::Shift(b, i) => {
                return self.simplify_inner(b, &s.shift(i.min(s.len())), var_map);
            }
            CheckedPrismExpr::TypeAssert(e, typ) => {
                let e = self.simplify_inner(e, s, var_map);
                let typ = self.simplify_inner(typ, s, var_map);
                CheckedPrismExpr::TypeAssert(e, typ)
            }
            CheckedPrismExpr::GrammarValue(p) => CheckedPrismExpr::GrammarValue(p),
            CheckedPrismExpr::GrammarType => CheckedPrismExpr::GrammarType,
        };
        self.store_checked(e_new, self.checked_origins[*i])
    }
}
