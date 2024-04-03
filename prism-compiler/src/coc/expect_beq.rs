use std::collections::HashMap;
use crate::coc::env::{Env, UniqueVariableId};
use crate::coc::env::EnvEntry::*;
use crate::coc::{PartialExpr, TcEnv};
use crate::coc::type_check::TcError;
use crate::coc::UnionIndex;

impl TcEnv {
    ///Invariant: `a` and `b` are valid in `s`
    pub fn expect_beq(&mut self, i1: UnionIndex, i2: UnionIndex, s: &Env, errors: &mut Vec<TcError>) {
        self.expect_beq_internal((i1, s, &mut HashMap::new()), (i2, s, &mut HashMap::new()), errors)
    }

    ///Invariant: `a` and `b` are valid in `s`
    fn expect_beq_internal(&mut self, (i1, s1, var_map1): (UnionIndex, &Env, &mut HashMap<UniqueVariableId, usize>), (i2, s2, var_map2): (UnionIndex, &Env, &mut HashMap<UniqueVariableId, usize>), errors: &mut Vec<TcError>) {
        // Brh and reduce i1 and i2
        let (i1, s1) = self.beta_reduce_head(i1, s1.clone());
        let (i2, s2) = self.beta_reduce_head(i2, s2.clone());

        match (self.values[i1.0], self.values[i2.0]) {
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
                    errors.push(());
                }
            }
            (PartialExpr::FnType(a1, b1), PartialExpr::FnType(a2, b2)) => {
                self.expect_beq_internal((a1, &s1, var_map1), (a2, &s2, var_map2), errors);
                let id = self.new_tc_id();
                var_map1.insert(id, s1.len());
                var_map2.insert(id, s2.len());
                self.expect_beq_internal((b1, &s1.cons(RType(id)), var_map1), (b2, &s2.cons(RType(id)), var_map2), errors);
            }
            (PartialExpr::FnConstruct(a1, b1), PartialExpr::FnConstruct(a2, b2)) => {
                self.expect_beq_internal((a1, &s1, var_map1), (a2, &s2, var_map2), errors);
                let id = self.new_tc_id();
                var_map1.insert(id, s1.len());
                var_map2.insert(id, s2.len());
                self.expect_beq_internal((b1, &s1.cons(RType(id)), var_map1), (b2, &s2.cons(RType(id)), var_map2), errors);
            }
            (PartialExpr::FnDestruct(f1, a1), PartialExpr::FnDestruct(f2, a2)) => {
                self.expect_beq_internal((f1, &s1, var_map1), (f2, &s2, var_map2), errors);
                self.expect_beq_internal((a1, &s1, var_map1), (a2, &s2, var_map2), errors);
            }
            (PartialExpr::Free, PartialExpr::Free) => {
                //TODO queue this constraint
                todo!()
            }
            (e1, PartialExpr::Free) => {
                self.expect_beq_free((i1, e1, &s1, var_map1), (i2, &s2, var_map2), errors);
            }
            (PartialExpr::Free, e2) => {
                self.expect_beq_free((i2, e2, &s2, var_map2), (i1, &s1, var_map1), errors);
            }
            (_e1, _e2) => {
                errors.push(());
            }
        }
    }

    // i2 should be free
    fn expect_beq_free(&mut self, (i1, e1, s1, var_map1): (UnionIndex, PartialExpr, &Env, &mut HashMap<UniqueVariableId, usize>), (i2, s2, var_map2): (UnionIndex, &Env, &mut HashMap<UniqueVariableId, usize>), errors: &mut Vec<TcError>) {
        match e1 {
            PartialExpr::Type => {
                self.values[i2.0] = PartialExpr::Type
            }
            PartialExpr::Var(v1) => {
                self.values[i2.0] = match s1[v1] {
                    CType(id, _) => {
                        let v2 = v1 + s2.len() - s1.len();

                        // Sanity check
                        let CType(id2, _) = s2[v2] else {
                            panic!("Sanity check failed")
                        };
                        assert_eq!(id, id2);

                        PartialExpr::Var(v2)
                    }
                    RType(id) => {
                        let v2 = s2.len() - var_map2[&id] - 1;

                        // Sanity check
                        let RType(id2) = s2[v2] else {
                            panic!("Sanity check failed")
                        };
                        assert_eq!(id, id2);

                        PartialExpr::Var(v2)
                    }
                    RSubst(_, _) | CSubst(_, _) => unreachable!(),
                }
            }
            PartialExpr::FnType(_, _) => {
                self.values[i2.0] = PartialExpr::FnType(self.insert_union_index(PartialExpr::Free), self.insert_union_index(PartialExpr::Free));
                self.expect_beq_internal((i1, s1, var_map1), (i2, s2, var_map2), errors);
            }
            PartialExpr::FnConstruct(_, _) => {
                self.values[i2.0] = PartialExpr::FnConstruct(self.insert_union_index(PartialExpr::Free), self.insert_union_index(PartialExpr::Free));
                self.expect_beq_internal((i1, s1, var_map1), (i2, s2, var_map2), errors);
            }
            PartialExpr::FnDestruct(_, _) => {
                self.values[i2.0] = PartialExpr::FnDestruct(self.insert_union_index(PartialExpr::Free), self.insert_union_index(PartialExpr::Free));
                self.expect_beq_internal((i1, s1, var_map1), (i2, s2, var_map2), errors);
            }
            PartialExpr::Shift(_, _) | PartialExpr::Let(_, _) | PartialExpr::Free => unreachable!(),
        }
    }
}
