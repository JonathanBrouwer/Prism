use crate::coc::{PartialExpr, TcEnv};
use crate::coc::env::Env;
use crate::coc::env::EnvEntry::*;
use crate::union_find::UnionIndex;

impl TcEnv {
    pub fn beq(
        &mut self, i1: UnionIndex, s1: &Env, i2: UnionIndex, s2: &Env
    ) -> bool {
        // Brh and reduce i1 and i2
        let (i1, s1) = self.brh(i1, s1.clone());
        let (i2, s2) = self.brh(i2, s2.clone());
        let i1 = self.uf.find(i1);
        let i2 = self.uf.find(i2);

        match (&self.uf_values[i1.0], &self.uf_values[i2.0]) {
            (&PartialExpr::Type, &PartialExpr::Type) => {}
            (&PartialExpr::Var(i1), &PartialExpr::Var(i2)) => {
                let i1 = i1 + s2.len();
                let i2 = i2 + s1.len();
                if i1 != i2 { return false; }
            }
            (&PartialExpr::FnType(a1, b1), &PartialExpr::FnType(a2, b2)) => {
                if !self.beq(a1, &s1, a2, &s2) { return false; }
                if !self.beq(b1, &s1.cons(RType), b2, &s2.cons(RType)) { return false; }
            }
            (&PartialExpr::FnConstruct(a1, b1), &PartialExpr::FnConstruct(a2, b2)) => {
                if !self.beq(a1, &s1, a2, &s2) { return false; }
                if !self.beq(b1, &s1.cons(RType), b2, &s2.cons(RType)) { return false; }
            }
            (&PartialExpr::FnDestruct(f1, a1), &PartialExpr::FnDestruct(f2, a2)) => {
                if !self.beq(f1, &s1, f2, &s2) { return false; }
                if !self.beq(a1, &s1, a2, &s2) { return false; }
            }
            _ => {
                return false;
            }
        }
        
        return true;
    }
}
