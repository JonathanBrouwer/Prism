use crate::lang::env::EnvEntry::*;
use crate::lang::env::{Env, UniqueVariableId};
use crate::lang::error::TypeError;
use crate::lang::UnionIndex;
use crate::lang::ValueOrigin::FreeSub;
use crate::lang::{PartialExpr, TcEnv};
use std::collections::HashMap;

impl TcEnv {
    #[must_use]
    pub fn expect_beq_internal(
        &mut self,
        (i1o, s1, var_map1): (UnionIndex, &Env, &mut HashMap<UniqueVariableId, usize>),
        (i2o, s2, var_map2): (UnionIndex, &Env, &mut HashMap<UniqueVariableId, usize>),
    ) -> bool {
        // Brh and reduce i1 and i2
        let (i1, s1) = self.beta_reduce_head(i1o, s1.clone());
        let (i2, s2) = self.beta_reduce_head(i2o, s2.clone());

        match (self.values[i1.0], self.values[i2.0]) {
            (PartialExpr::Type, PartialExpr::Type) => {
                // If beta_reduce returns a Type, we're done. Easy work!
                true
            }
            (PartialExpr::DeBruijnIndex(index1), PartialExpr::DeBruijnIndex(index2)) => {
                let id1 = match s1[index1] {
                    CType(id, _) | RType(id) => id,
                    CSubst(..) | RSubst(..) => unreachable!(),
                };
                let id2 = match s2[index2] {
                    CType(id, _) | RType(id) => id,
                    CSubst(..) | RSubst(..) => unreachable!(),
                };
                id1 == id2
            }
            (PartialExpr::FnType(a1, b1), PartialExpr::FnType(a2, b2)) => {
                let a_equal = self.expect_beq_internal((a1, &s1, var_map1), (a2, &s2, var_map2));
                let id = self.new_tc_id();
                var_map1.insert(id, s1.len());
                var_map2.insert(id, s2.len());
                let b_equal = self.expect_beq_internal(
                    (b1, &s1.cons(RType(id)), var_map1),
                    (b2, &s2.cons(RType(id)), var_map2),
                );
                a_equal && b_equal
            }
            (PartialExpr::FnConstruct(a1, b1), PartialExpr::FnConstruct(a2, b2)) => {
                let a_equal = self.expect_beq_internal((a1, &s1, var_map1), (a2, &s2, var_map2));
                let id = self.new_tc_id();
                var_map1.insert(id, s1.len());
                var_map2.insert(id, s2.len());
                let b_equal = self.expect_beq_internal(
                    (b1, &s1.cons(RType(id)), var_map1),
                    (b2, &s2.cons(RType(id)), var_map2),
                );
                a_equal && b_equal
            }
            (PartialExpr::FnDestruct(f1, a1), PartialExpr::FnDestruct(f2, a2)) => {
                let f_equal = self.expect_beq_internal((f1, &s1, var_map1), (f2, &s2, var_map2));
                let b_equal = self.expect_beq_internal((a1, &s1, var_map1), (a2, &s2, var_map2));
                f_equal && b_equal
            }
            (_, PartialExpr::Free) => {
                self.expect_beq_free((i1, &s1, var_map1), (i2, &s2, var_map2))
            }
            (PartialExpr::Free, _) => {
                self.expect_beq_free((i2, &s2, var_map2), (i1, &s1, var_map1))
            }
            _ => false,
        }
    }

    /// Precondition: i2 should be free
    #[must_use]
    pub fn expect_beq_free(
        &mut self,
        (i1, s1, var_map1): (UnionIndex, &Env, &mut HashMap<UniqueVariableId, usize>),
        (i2, s2, var_map2): (UnionIndex, &Env, &mut HashMap<UniqueVariableId, usize>),
    ) -> bool {
        debug_assert!(matches!(self.values[i2.0], PartialExpr::Free));

        if self.toxic_values.contains(&i1) {
            self.errors.push(TypeError::InfiniteType(i1, i2));
            return true;
        }

        return match self.values[i1.0] {
            PartialExpr::Type => {
                self.values[i2.0] = PartialExpr::Type;
                self.handle_constraints(i2, s2)
            }
            PartialExpr::Let(v1, b1) => self.expect_beq_free(
                (b1, &s1.cons(RSubst(v1, s1.clone())), var_map1),
                (i2, s2, var_map2),
            ),
            PartialExpr::DeBruijnIndex(v1) => {
                let subst_equal = match &s1[v1] {
                    &CType(id, _) => {
                        // We may have shifted away part of the env that we need during this beq
                        let Some(v2) = (v1 + s2.len()).checked_sub(s1.len()) else {
                            self.errors.push(TypeError::BadInfer {
                                free_var: i2,
                                inferred_var: i1,
                            });
                            return true;
                        };
                        let CType(id2, _) = s2[v2] else {
                            self.errors.push(TypeError::BadInfer {
                                free_var: i2,
                                inferred_var: i1,
                            });
                            return true;
                        };
                        // Sanity check, after the correct value is shifted away it should not be possible for another C value to reappear
                        debug_assert_eq!(id, id2);

                        self.values[i2.0] = PartialExpr::DeBruijnIndex(v2);
                        true
                    }
                    &CSubst(_, _) => {
                        // Same story as above, except we don't have the `id` to double check with here.
                        // The logic should still hold even without the sanity check though
                        let Some(v2) = (v1 + s2.len()).checked_sub(s1.len()) else {
                            self.errors.push(TypeError::BadInfer {
                                free_var: i2,
                                inferred_var: i1,
                            });
                            return true;
                        };
                        let CSubst(_, _) = s2[v2] else {
                            self.errors.push(TypeError::BadInfer {
                                free_var: i2,
                                inferred_var: i1,
                            });
                            return true;
                        };

                        self.values[i2.0] = PartialExpr::DeBruijnIndex(v2);
                        true
                    }
                    &RType(id) => {
                        // If `id` still exists in s2, we will find it here
                        let Some(v2) = s2.len().checked_sub(var_map2[&id] + 1) else {
                            self.errors.push(TypeError::BadInfer {
                                free_var: i2,
                                inferred_var: i1,
                            });
                            return true;
                        };
                        // Check if it still exists, if not we shifted it out and another entry came in the place for it
                        let RType(id2) = s2[v2] else {
                            self.errors.push(TypeError::BadInfer {
                                free_var: i2,
                                inferred_var: i1,
                            });
                            panic!("I think this can happen but I want an example for it");
                            // TODO return true;
                        };
                        if id != id2 {
                            self.errors.push(TypeError::BadInfer {
                                free_var: i2,
                                inferred_var: i1,
                            });
                            panic!("I think this can happen but I want an example for it");
                            // TODO return true;
                        }

                        self.values[i2.0] = PartialExpr::DeBruijnIndex(v2);
                        true
                    }
                    RSubst(i1, s1) => self.expect_beq_free((*i1, s1, var_map1), (i2, s2, var_map2)),
                };
                let constraints_eq = self.handle_constraints(i2, s2);
                subst_equal && constraints_eq
            }
            PartialExpr::FnType(a1, b1) => {
                let a2 = self.store(PartialExpr::Free, FreeSub(i2));
                let b2 = self.store(PartialExpr::Free, FreeSub(i2));
                self.values[i2.0] = PartialExpr::FnType(a2, b2);

                let constraints_eq = self.handle_constraints(i2, s2);

                self.toxic_values.insert(i2);
                let a_eq = self.expect_beq_free((a1, s1, var_map1), (a2, s2, var_map2));
                let id = self.new_tc_id();
                var_map1.insert(id, s1.len());
                var_map2.insert(id, s2.len());
                let b_eq = self.expect_beq_free(
                    (b1, &s1.cons(RType(id)), var_map1),
                    (b2, &s2.cons(RType(id)), var_map2),
                );

                constraints_eq && a_eq && b_eq
            }
            PartialExpr::FnConstruct(a1, b1) => {
                let a2 = self.store(PartialExpr::Free, FreeSub(i2));
                let b2 = self.store(PartialExpr::Free, FreeSub(i2));
                self.values[i2.0] = PartialExpr::FnConstruct(a2, b2);

                let constraints_eq = self.handle_constraints(i2, s2);

                self.toxic_values.insert(i2);
                let a_eq = self.expect_beq_free((a1, s1, var_map1), (a2, s2, var_map2));

                let id = self.new_tc_id();
                var_map1.insert(id, s1.len());
                var_map2.insert(id, s2.len());
                let b_eq = self.expect_beq_free(
                    (b1, &s1.cons(RType(id)), var_map1),
                    (b2, &s2.cons(RType(id)), var_map2),
                );

                constraints_eq && a_eq && b_eq
            }
            PartialExpr::FnDestruct(f1, a1) => {
                let f2 = self.store(PartialExpr::Free, FreeSub(i2));
                let a2 = self.store(PartialExpr::Free, FreeSub(i2));
                self.values[i2.0] = PartialExpr::FnDestruct(f2, a2);

                let constraints_eq = self.handle_constraints(i2, s2);

                self.toxic_values.insert(i2);
                let f_eq = self.expect_beq_free((f1, s1, var_map1), (f2, s2, var_map2));
                let a_eq = self.expect_beq_free((a1, s1, var_map1), (a2, s2, var_map2));
                constraints_eq && f_eq && a_eq
            }
            PartialExpr::Free => {
                self.queued_beq_free.entry(i1).or_default().push((
                    (s1.clone(), var_map1.clone()),
                    (i2, s2.clone(), var_map2.clone()),
                ));
                self.queued_beq_free.entry(i2).or_default().push((
                    (s2.clone(), var_map2.clone()),
                    (i1, s1.clone(), var_map1.clone()),
                ));
                true
            }
            PartialExpr::Shift(v1, i) => {
                self.expect_beq_free((v1, &s1.shift(i), var_map1), (i2, s2, var_map2))
            }
        };
    }

    #[must_use]
    pub fn handle_constraints(&mut self, i2: UnionIndex, s2: &Env) -> bool {
        let mut eq = true;

        // Check queued constraints
        if let Some(queued) = self.queued_beq_free.remove(&i2) {
            for ((s2n, mut var_map2n), (i3, s3, mut var_map3)) in queued {
                // Sanity checks
                debug_assert_eq!(s2.len(), s2n.len());

                //TODO performance: this causes expect_beq(i3, i2) to be executed
                eq &=
                    self.expect_beq_internal((i2, &s2n, &mut var_map2n), (i3, &s3, &mut var_map3));
            }
        }

        // if let Some((s, t2)) = self.queued_tc.remove(&i2) {
        //     let t1 = self._type_check(i2, &s);
        //     eq &= self.expect_beq_internal((t1, &s, &mut HashMap::new()), (t2, &s, &mut HashMap::new()));
        // }

        eq
    }
}
