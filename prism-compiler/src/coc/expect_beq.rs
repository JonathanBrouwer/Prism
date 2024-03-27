use crate::coc::env::Env;
use crate::coc::env::EnvEntry::*;
use crate::coc::{PartialExpr, TcEnv};
use crate::union_find::UnionIndex;

impl TcEnv {
    ///Invariant: `a` is valid in `s`
    pub fn expect_beq_type(&mut self, i: UnionIndex, s: &Env) {
        self.expect_beq(i, Self::type_type(), s)
    }

    ///Invariant: `a` and `b` are valid in `s`
    pub fn expect_beq(&mut self, i1: UnionIndex, i2: UnionIndex, s: &Env) {
        self.expect_beq_internal(i1, s, i2, s)
    }

    ///Invariant: `a` and `b` are valid in `s`
    fn expect_beq_internal(&mut self, i1: UnionIndex, s1: &Env, i2: UnionIndex, s2: &Env) {
        // Brh and reduce i1 and i2
        let (i1, s1) = self.beta_reduce_head(i1, s1.clone());
        let (i2, s2) = self.beta_reduce_head(i2, s2.clone());
        let i1 = self.uf.find(i1);
        let i2 = self.uf.find(i2);

        match (self.uf_values[i1.0], self.uf_values[i2.0]) {
            (PartialExpr::Type, PartialExpr::Type) => {
                // If beta_reduce returns a Type, we're done. Easy work!
            }
            (PartialExpr::Var(i1), PartialExpr::Var(i2)) => {
                let id1 = match s1[i1] {
                    CType(id, _) | RType(id) => id,
                    CSubst(..) | RSubst(..) => unreachable!(),
                };
                let id2 = match s2[i2] {
                    CType(id, _) | RType(id) => id,
                    CSubst(..) | RSubst(..) => unreachable!(),
                };
                if id1 != id2 {
                    self.errors.push(());
                }
            }
            (PartialExpr::FnType(a1, b1), PartialExpr::FnType(a2, b2)) => {
                self.expect_beq_internal(a1, &s1, a2, &s2);
                let id = self.new_tc_id();
                self.expect_beq_internal(b1, &s1.cons(RType(id)), b2, &s2.cons(RType(id)));
            }
            (PartialExpr::FnConstruct(a1, b1), PartialExpr::FnConstruct(a2, b2)) => {
                self.expect_beq_internal(a1, &s1, a2, &s2);
                let id = self.new_tc_id();
                self.expect_beq_internal(b1, &s1.cons(RType(id)), b2, &s2.cons(RType(id)));
            }
            (PartialExpr::FnDestruct(f1, a1), PartialExpr::FnDestruct(f2, a2)) => {
                self.expect_beq_internal(f1, &s1, f2, &s2);
                self.expect_beq_internal(a1, &s1, a2, &s2);
            }
            (_e, PartialExpr::Free) => {
                //TODO this is incorrect, need to capture environment -> 
                // - Traverse left side, and fill in right side based on that. 
                // - Need a hashmap to keep track of UniqueId -> Index
                // - For free/free case, need to wait with solving until later. Maybe keep track of all frees?

                // self.uf.union_left(i1, i2);
            }
            (PartialExpr::Free, _e) => {
                //TODO this is also incorrect
                // self.uf.union_left(i2, i1);
            }
            (_e1, _e2) => {
                self.errors.push(());
            }
        }
    }
}
