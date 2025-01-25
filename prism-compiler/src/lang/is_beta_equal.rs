use crate::lang::UnionIndex;
use crate::lang::env::Env;
use crate::lang::env::EnvEntry::*;
use crate::lang::{PrismEnv, PrismExpr};

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn is_beta_equal(&mut self, i1: UnionIndex, s1: &Env, i2: UnionIndex, s2: &Env) -> bool {
        // Brh and reduce i1 and i2
        let (i1, s1) = self.beta_reduce_head(i1, s1.clone());
        let (i2, s2) = self.beta_reduce_head(i2, s2.clone());

        match (self.values[*i1], self.values[*i2]) {
            (PrismExpr::Type, PrismExpr::Type) => {}
            (PrismExpr::DeBruijnIndex(i1), PrismExpr::DeBruijnIndex(i2)) => {
                let id1 = match s1[i1] {
                    CType(id, _, _) | RType(id) => id,
                    CSubst(..) | RSubst(..) => unreachable!(),
                };
                let id2 = match s2[i2] {
                    CType(id, _, _) | RType(id) => id,
                    CSubst(..) | RSubst(..) => unreachable!(),
                };
                if id1 != id2 {
                    return false;
                }
            }
            (PrismExpr::FnType(_, a1, b1), PrismExpr::FnType(_, a2, b2)) => {
                if !self.is_beta_equal(a1, &s1, a2, &s2) {
                    return false;
                }
                let id = self.new_tc_id();
                if !self.is_beta_equal(b1, &s1.cons(RType(id)), b2, &s2.cons(RType(id))) {
                    return false;
                }
            }
            (PrismExpr::FnConstruct(_, b1), PrismExpr::FnConstruct(_, b2)) => {
                let id = self.new_tc_id();
                if !self.is_beta_equal(b1, &s1.cons(RType(id)), b2, &s2.cons(RType(id))) {
                    return false;
                }
            }
            (PrismExpr::FnDestruct(f1, a1), PrismExpr::FnDestruct(f2, a2)) => {
                if !self.is_beta_equal(f1, &s1, f2, &s2) {
                    return false;
                }
                if !self.is_beta_equal(a1, &s1, a2, &s2) {
                    return false;
                }
            }
            (PrismExpr::Free, PrismExpr::Free) => {}
            _ => {
                return false;
            }
        }

        true
    }
}
