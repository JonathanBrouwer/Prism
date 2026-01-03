use crate::lang::CorePrismExpr;
use crate::lang::env::{DbEnv, EnvEntry};
use crate::lang::{CoreIndex, PrismDb};
use crate::type_check::{TypecheckPrismEnv, UniqueVariableId};
use std::collections::HashMap;

impl PrismDb {
    pub fn simplify(&mut self, i: CoreIndex) -> CoreIndex {
        let mut env = TypecheckPrismEnv::new(self);
        env.simplify_inner(i, &DbEnv::default(), &mut HashMap::new())
    }
}

impl TypecheckPrismEnv<'_> {
    fn simplify_inner(
        &mut self,
        i: CoreIndex,
        s: &DbEnv,
        var_map: &mut HashMap<UniqueVariableId, usize>,
    ) -> CoreIndex {
        let e_new = match &self.db.checked_values[*i] {
            CorePrismExpr::Type => CorePrismExpr::Type,
            &CorePrismExpr::Let(v, b) => {
                let v = self.simplify_inner(v, s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                CorePrismExpr::Let(v, b)
            }
            &CorePrismExpr::DeBruijnIndex(v) => match s.get_idx(v) {
                Some(EnvEntry::CType(_, _)) | Some(EnvEntry::CSubst(_, _)) => unreachable!(),
                Some(EnvEntry::RType(id)) => {
                    CorePrismExpr::DeBruijnIndex(var_map.len() - var_map[id] - 1)
                }
                Some(EnvEntry::RSubst(subst, subst_env)) => {
                    return self.simplify_inner(*subst, subst_env, var_map);
                }
                None => CorePrismExpr::DeBruijnIndex(v),
            },
            &CorePrismExpr::FnType(a, b) => {
                let a = self.simplify_inner(a, s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                CorePrismExpr::FnType(a, b)
            }
            &CorePrismExpr::FnConstruct(b) => {
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                CorePrismExpr::FnConstruct(b)
            }
            &CorePrismExpr::FnDestruct(a, b) => {
                let a = self.simplify_inner(a, s, var_map);
                let b = self.simplify_inner(b, s, var_map);
                CorePrismExpr::FnDestruct(a, b)
            }
            CorePrismExpr::Free => CorePrismExpr::Free,
            &CorePrismExpr::Shift(b, i) => {
                return self.simplify_inner(b, &s.shift(i.min(s.len())), var_map);
            }
            &CorePrismExpr::TypeAssert(e, typ) => {
                let e = self.simplify_inner(e, s, var_map);
                let typ = self.simplify_inner(typ, s, var_map);
                CorePrismExpr::TypeAssert(e, typ)
            }
            CorePrismExpr::GrammarValue(p) => CorePrismExpr::GrammarValue(p.clone()),
            CorePrismExpr::GrammarType => CorePrismExpr::GrammarType,
        };
        self.db.store_checked(e_new, self.db.checked_origins[*i])
    }
}
