use crate::coc::env::Env;
use crate::coc::env::EnvEntry::*;
use crate::coc::{PartialExpr, TcEnv};
use crate::union_find::UnionIndex;

impl TcEnv {
    pub fn beq(&mut self, i1: UnionIndex, s1: &Env, i2: UnionIndex, s2: &Env) -> bool {
        // Brh and reduce i1 and i2
        let (i1, s1) = self.brh(i1, s1.clone());
        let (i2, s2) = self.brh(i2, s2.clone());
        let i1 = self.uf.find(i1);
        let i2 = self.uf.find(i2);

        match (&self.uf_values[i1.0], &self.uf_values[i2.0]) {
            (&PartialExpr::Type, &PartialExpr::Type) => {}
            (&PartialExpr::Var(i1), &PartialExpr::Var(i2)) => {
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
            (&PartialExpr::FnType(a1, b1), &PartialExpr::FnType(a2, b2)) => {
                if !self.beq(a1, &s1, a2, &s2) {
                    return false;
                }
                let id = self.new_tc_id();
                if !self.beq(b1, &s1.cons(RType(id)), b2, &s2.cons(RType(id))) {
                    return false;
                }
            }
            (&PartialExpr::FnConstruct(a1, b1), &PartialExpr::FnConstruct(a2, b2)) => {
                if !self.beq(a1, &s1, a2, &s2) {
                    return false;
                }
                let id = self.new_tc_id();
                if !self.beq(b1, &s1.cons(RType(id)), b2, &s2.cons(RType(id))) {
                    return false;
                }
            }
            (&PartialExpr::FnDestruct(f1, a1), &PartialExpr::FnDestruct(f2, a2)) => {
                if !self.beq(f1, &s1, f2, &s2) {
                    return false;
                }
                if !self.beq(a1, &s1, a2, &s2) {
                    return false;
                }
            }
            _ => {
                return false;
            }
        }

        return true;
    }
}
