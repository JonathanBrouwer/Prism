use crate::lang::CheckedIndex;
use crate::lang::env::DbEnv;
use crate::lang::env::EnvEntry::*;
use crate::lang::{CheckedPrismExpr, PrismEnv};

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn is_beta_equal(
        &mut self,
        i1: CheckedIndex,
        s1: DbEnv,
        i2: CheckedIndex,
        s2: DbEnv,
    ) -> bool {
        // Brh and reduce i1 and i2
        let (i1, s1) = self.beta_reduce_head(i1, s1.clone());
        let (i2, s2) = self.beta_reduce_head(i2, s2.clone());

        match (self.checked_values[*i1], self.checked_values[*i2]) {
            (CheckedPrismExpr::Type, CheckedPrismExpr::Type) => {}
            (CheckedPrismExpr::GrammarType, CheckedPrismExpr::GrammarType) => {}
            (CheckedPrismExpr::DeBruijnIndex(i1), CheckedPrismExpr::DeBruijnIndex(i2)) => {
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
            (CheckedPrismExpr::FnType(a1, b1), CheckedPrismExpr::FnType(a2, b2)) => {
                if !self.is_beta_equal(a1, s1, a2, s2) {
                    return false;
                }
                let id = self.new_tc_id();
                if !self.is_beta_equal(
                    b1,
                    s1.cons(RType(id), self.allocs),
                    b2,
                    s2.cons(RType(id), self.allocs),
                ) {
                    return false;
                }
            }
            (CheckedPrismExpr::FnConstruct(b1), CheckedPrismExpr::FnConstruct(b2)) => {
                let id = self.new_tc_id();
                if !self.is_beta_equal(
                    b1,
                    s1.cons(RType(id), self.allocs),
                    b2,
                    s2.cons(RType(id), self.allocs),
                ) {
                    return false;
                }
            }
            (CheckedPrismExpr::FnDestruct(f1, a1), CheckedPrismExpr::FnDestruct(f2, a2)) => {
                if !self.is_beta_equal(f1, s1, f2, s2) {
                    return false;
                }
                if !self.is_beta_equal(a1, s1, a2, s2) {
                    return false;
                }
            }
            (CheckedPrismExpr::Free, CheckedPrismExpr::Free) => {}
            _ => {
                return false;
            }
        }

        true
    }
}
