use crate::lang::CorePrismExpr;
use crate::lang::env::DbEnv;
use crate::lang::env::EnvEntry::*;
use crate::lang::{CoreIndex, PrismDb};
use crate::type_check::TypecheckPrismEnv;

impl PrismDb {
    pub fn is_beta_equal(&mut self, i1: CoreIndex, s1: &DbEnv, i2: CoreIndex, s2: &DbEnv) -> bool {
        let mut env = TypecheckPrismEnv::new(self);
        env.is_beta_equal(i1, s1, i2, s2)
    }
}

impl TypecheckPrismEnv<'_> {
    fn is_beta_equal(&mut self, i1: CoreIndex, s1: &DbEnv, i2: CoreIndex, s2: &DbEnv) -> bool {
        // Brh and reduce i1 and i2
        let (i1, s1) = self.db.beta_reduce_head(i1, s1);
        let (i2, s2) = self.db.beta_reduce_head(i2, s2);

        match (&self.db.values[*i1], &self.db.values[*i2]) {
            (CorePrismExpr::Type, CorePrismExpr::Type) => {}
            (CorePrismExpr::GrammarType, CorePrismExpr::GrammarType) => {}
            (&CorePrismExpr::DeBruijnIndex(i1), &CorePrismExpr::DeBruijnIndex(i2)) => {
                let id1 = match s1[i1] {
                    CType(id, _) | RType(id) => id,
                    CSubst(..) | RSubst(..) => unreachable!(),
                };
                let id2 = match s2[i2] {
                    CType(id, _) | RType(id) => id,
                    CSubst(..) | RSubst(..) => unreachable!(),
                };
                if id1 != id2 {
                    return false;
                }
            }
            (&CorePrismExpr::FnType(a1, b1), &CorePrismExpr::FnType(a2, b2)) => {
                if !self.is_beta_equal(a1, &s1, a2, &s2) {
                    return false;
                }
                let id = self.new_tc_id();
                if !self.is_beta_equal(b1, &s1.cons(RType(id)), b2, &s2.cons(RType(id))) {
                    return false;
                }
            }
            (&CorePrismExpr::FnConstruct(b1), &CorePrismExpr::FnConstruct(b2)) => {
                let id = self.new_tc_id();
                if !self.is_beta_equal(b1, &s1.cons(RType(id)), b2, &s2.cons(RType(id))) {
                    return false;
                }
            }
            (&CorePrismExpr::FnDestruct(f1, a1), &CorePrismExpr::FnDestruct(f2, a2)) => {
                if !self.is_beta_equal(f1, &s1, f2, &s2) {
                    return false;
                }
                if !self.is_beta_equal(a1, &s1, a2, &s2) {
                    return false;
                }
            }
            (CorePrismExpr::Free, CorePrismExpr::Free) => {}
            _ => {
                return false;
            }
        }

        true
    }
}
