use crate::lang::CoreIndex;
use crate::lang::ValueOrigin::FreeSub;
use crate::lang::env::EnvEntry::*;
use crate::lang::env::{DbEnv, UniqueVariableId};
use crate::lang::error::TypeError;
use crate::lang::{CorePrismExpr, PrismDb};
use std::collections::HashMap;

const MAX_BEQ_DEPTH: usize = 256;

impl PrismDb {
    #[must_use]
    pub fn expect_beq_internal(
        &mut self,
        // io is the UnionIndex that lives in a certain `s`
        // The var_map is a map for each `UniqueVariableId`, its depth in the scope
        (i1o, s1, var_map1): (CoreIndex, &DbEnv, &mut HashMap<UniqueVariableId, usize>),
        (i2o, s2, var_map2): (CoreIndex, &DbEnv, &mut HashMap<UniqueVariableId, usize>),
        depth: usize,
    ) -> bool {
        if depth > MAX_BEQ_DEPTH {
            self.push_type_error(TypeError::RecursionLimit(i1o, i2o));
            return true;
        }

        // Brh and reduce i1 and i2
        let (i1, s1) = self.beta_reduce_head(i1o, s1);
        let (i2, s2) = self.beta_reduce_head(i2o, s2);

        match (&self.checked_values[*i1], &self.checked_values[*i2]) {
            // Type is always equal to Type
            (CorePrismExpr::Type, CorePrismExpr::Type) => {
                // If beta_reduce returns a Type, we're done. Easy work!
                true
            }
            // ParsedType is always equal to ParsedType
            (CorePrismExpr::GrammarType, CorePrismExpr::GrammarType) => true,
            // Two de bruijn indices are equal if they refer to the same `CType` or `RType` (function argument)
            // Because `i1` and `i2` live in a different scope, the equality of `index1` and `index2` needs to be retrieved from the scope
            (&CorePrismExpr::DeBruijnIndex(index1), &CorePrismExpr::DeBruijnIndex(index2)) => {
                // Get the UniqueVariableId that `index1` refers to
                let id1 = match s1[index1] {
                    CType(id, _) | RType(id) => id,
                    CSubst(..) | RSubst(..) => unreachable!(),
                };
                // Get the UniqueVariableId that `index2` refers to
                let id2 = match s2[index2] {
                    CType(id, _) | RType(id) => id,
                    CSubst(..) | RSubst(..) => unreachable!(),
                };
                // Check if the unique variable indices (not the de bruijn indices) are equal
                id1 == id2
            }
            // Two function types are equal if their argument types and body types are equal
            (&CorePrismExpr::FnType(a1, b1), &CorePrismExpr::FnType(a2, b2)) => {
                // Check that the argument types are equal
                let a_equal =
                    self.expect_beq_internal((a1, &s1, var_map1), (a2, &s2, var_map2), depth + 1);

                // Insert the new variable into the scopes, and check if `b` is equal
                let id = self.new_tc_id();
                var_map1.insert(id, s1.len());
                var_map2.insert(id, s2.len());
                let b_equal = self.expect_beq_internal(
                    (b1, &s1.cons(RType(id)), var_map1),
                    (b2, &s2.cons(RType(id)), var_map2),
                    depth + 1,
                );

                // Function types are equal if the arguments and body are equal
                a_equal && b_equal
            }
            // Function construct works the same as above
            (&CorePrismExpr::FnConstruct(b1), &CorePrismExpr::FnConstruct(b2)) => {
                let id = self.new_tc_id();
                var_map1.insert(id, s1.len());
                var_map2.insert(id, s2.len());

                self.expect_beq_internal(
                    (b1, &s1.cons(RType(id)), var_map1),
                    (b2, &s2.cons(RType(id)), var_map2),
                    depth + 1,
                )
            }
            // Function destruct (application) is only equal if the functions and the argument are equal
            // This can only occur in this position when `f1` and `f2` are arguments to a function in the original scope
            (&CorePrismExpr::FnDestruct(f1, a1), &CorePrismExpr::FnDestruct(f2, a2)) => {
                let f_equal =
                    self.expect_beq_internal((f1, &s1, var_map1), (f2, &s2, var_map2), depth + 1);
                let b_equal =
                    self.expect_beq_internal((a1, &s1, var_map1), (a2, &s2, var_map2), depth + 1);
                f_equal && b_equal
            }
            (_, CorePrismExpr::Free) => {
                self.expect_beq_free((i1, &s1, var_map1), (i2, &s2, var_map2), depth + 1)
            }
            (CorePrismExpr::Free, _) => {
                self.expect_beq_free((i2, &s2, var_map2), (i1, &s1, var_map1), depth + 1)
            }
            (&CorePrismExpr::FnDestruct(f, _), _) => {
                self.expect_beq_in_destruct(f, &s1, var_map1, (i2, &s2, var_map2))
            }
            (_, &CorePrismExpr::FnDestruct(f, _)) => {
                self.expect_beq_in_destruct(f, &s2, var_map2, (i1, &s1, var_map1))
            }
            _ => false,
        }
    }

    pub fn expect_beq_in_destruct(
        &mut self,
        f1: CoreIndex,
        s1: &DbEnv,
        var_map1: &mut HashMap<UniqueVariableId, usize>,
        (i2, s2, var_map2): (CoreIndex, &DbEnv, &mut HashMap<UniqueVariableId, usize>),
    ) -> bool {
        let (f1, f1s) = self.beta_reduce_head(f1, s1);
        assert!(matches!(
            self.checked_values[*i2],
            CorePrismExpr::Type
                | CorePrismExpr::FnType(_, _)
                | CorePrismExpr::FnConstruct(_)
                | CorePrismExpr::DeBruijnIndex(_)
        ));

        // We are in the case `f1 a1 = i2`
        // This means the return value of `f1` must be `i2` (so `f1` ignores its argument)
        // We construct a value in scope 2 and set them equal
        let b = self.store_checked(CorePrismExpr::Shift(i2, 1), FreeSub(i2));
        let f = self.store_checked(CorePrismExpr::FnConstruct(b), FreeSub(i2));
        self.expect_beq_internal((f1, &f1s, var_map1), (f, s2, var_map2), 0)
    }

    /// Precondition: i2 should be free
    ///
    #[must_use]
    pub fn expect_beq_free(
        &mut self,
        (i1, s1, var_map1): (CoreIndex, &DbEnv, &mut HashMap<UniqueVariableId, usize>),
        (i2, s2, var_map2): (CoreIndex, &DbEnv, &mut HashMap<UniqueVariableId, usize>),
        depth: usize,
    ) -> bool {
        assert!(matches!(self.checked_values[*i2], CorePrismExpr::Free));
        if depth > MAX_BEQ_DEPTH {
            self.push_type_error(TypeError::RecursionLimit(i1, i2));
            return true;
        }

        // We deliberately don't beta-reduce i1 here since we want to keep the inferred value small
        match &self.checked_values[*i1] {
            CorePrismExpr::Type => {
                self.checked_values[*i2] = CorePrismExpr::Type;
                self.handle_constraints(i2, s2, depth + 1)
            }
            &CorePrismExpr::Let(v1, b1) => {
                let v2 = self.store_checked(CorePrismExpr::Free, FreeSub(i2));
                let b2 = self.store_checked(CorePrismExpr::Free, FreeSub(i2));
                self.checked_values[*i2] = CorePrismExpr::Let(v2, b2);

                let constraints_eq = self.handle_constraints(i2, s2, depth + 1);

                self.toxic_values.insert(i2);
                let a_eq =
                    self.expect_beq_internal((v1, s1, var_map1), (v2, s2, var_map2), depth + 1);

                let id = self.new_tc_id();
                var_map1.insert(id, s1.len());
                var_map2.insert(id, s2.len());
                let b_eq = self.expect_beq_internal(
                    (b1, &s1.cons(RType(id)), var_map1),
                    (b2, &s2.cons(RType(id)), var_map2),
                    depth + 1,
                );

                constraints_eq && a_eq && b_eq
            }
            &CorePrismExpr::DeBruijnIndex(v1) => {
                let subst_equal = match &s1[v1] {
                    &CType(id, _) => {
                        // We may have shifted away part of the env that we need during this beq
                        let Some(v2) = (v1 + s2.len()).checked_sub(s1.len()) else {
                            self.push_type_error(TypeError::BadInfer {
                                free_var: i2,
                                inferred_var: i1,
                            });
                            return true;
                        };
                        let CType(id2, _) = s2[v2] else {
                            self.push_type_error(TypeError::BadInfer {
                                free_var: i2,
                                inferred_var: i1,
                            });
                            return true;
                        };
                        // Sanity check, after the correct value is shifted away it should not be possible for another C value to reappear
                        assert_eq!(id, id2);

                        self.checked_values[*i2] = CorePrismExpr::DeBruijnIndex(v2);
                        true
                    }
                    &CSubst(_, _) => {
                        // Same story as above, except we don't have the `id` to double check with here.
                        // The logic should still hold even without the sanity check though
                        let Some(v2) = (v1 + s2.len()).checked_sub(s1.len()) else {
                            self.push_type_error(TypeError::BadInfer {
                                free_var: i2,
                                inferred_var: i1,
                            });
                            return true;
                        };
                        let CSubst(_, _) = s2[v2] else {
                            self.push_type_error(TypeError::BadInfer {
                                free_var: i2,
                                inferred_var: i1,
                            });
                            return true;
                        };

                        self.checked_values[*i2] = CorePrismExpr::DeBruijnIndex(v2);
                        true
                    }
                    &RType(id) => {
                        // If `id` still exists in s2, we will find it here
                        let Some(v2) = s2.len().checked_sub(var_map2[&id] + 1) else {
                            self.push_type_error(TypeError::BadInfer {
                                free_var: i2,
                                inferred_var: i1,
                            });
                            return true;
                        };
                        // Check if it still exists, if not we shifted it out and another entry came in the place for it
                        let RType(id2) = s2[v2] else {
                            self.push_type_error(TypeError::BadInfer {
                                free_var: i2,
                                inferred_var: i1,
                            });
                            panic!("I think this can happen but I want an example for it");
                            // TODO return true;
                        };
                        if id != id2 {
                            self.push_type_error(TypeError::BadInfer {
                                free_var: i2,
                                inferred_var: i1,
                            });
                            panic!("I think this can happen but I want an example for it");
                            // TODO return true;
                        }

                        self.checked_values[*i2] = CorePrismExpr::DeBruijnIndex(v2);
                        true
                    }
                    RSubst(i1, s1) => {
                        self.expect_beq_free((*i1, s1, var_map1), (i2, s2, var_map2), depth + 1)
                    }
                };
                let constraints_eq = self.handle_constraints(i2, s2, depth + 1);
                subst_equal && constraints_eq
            }
            &CorePrismExpr::FnType(a1, b1) => {
                let a2 = self.store_checked(CorePrismExpr::Free, FreeSub(i2));
                let b2 = self.store_checked(CorePrismExpr::Free, FreeSub(i2));
                self.checked_values[*i2] = CorePrismExpr::FnType(a2, b2);

                let constraints_eq = self.handle_constraints(i2, s2, depth + 1);

                self.toxic_values.insert(i2);
                let a_eq =
                    self.expect_beq_internal((a1, s1, var_map1), (a2, s2, var_map2), depth + 1);
                let id = self.new_tc_id();
                var_map1.insert(id, s1.len());
                var_map2.insert(id, s2.len());
                let b_eq = self.expect_beq_internal(
                    (b1, &s1.cons(RType(id)), var_map1),
                    (b2, &s2.cons(RType(id)), var_map2),
                    depth + 1,
                );

                constraints_eq && a_eq && b_eq
            }
            &CorePrismExpr::FnConstruct(b1) => {
                let b2 = self.store_checked(CorePrismExpr::Free, FreeSub(i2));
                self.checked_values[*i2] = CorePrismExpr::FnConstruct(b2);

                let constraints_eq = self.handle_constraints(i2, s2, depth + 1);

                self.toxic_values.insert(i2);

                let id = self.new_tc_id();
                var_map1.insert(id, s1.len());
                var_map2.insert(id, s2.len());
                let b_eq = self.expect_beq_internal(
                    (b1, &s1.cons(RType(id)), var_map1),
                    (b2, &s2.cons(RType(id)), var_map2),
                    depth + 1,
                );

                constraints_eq && b_eq
            }
            &CorePrismExpr::FnDestruct(f1, a1) => {
                let f2 = self.store_checked(CorePrismExpr::Free, FreeSub(i2));
                let a2 = self.store_checked(CorePrismExpr::Free, FreeSub(i2));
                self.checked_values[*i2] = CorePrismExpr::FnDestruct(f2, a2);

                let constraints_eq = self.handle_constraints(i2, s2, depth + 1);

                self.toxic_values.insert(i2);
                let f_eq =
                    self.expect_beq_internal((f1, s1, var_map1), (f2, s2, var_map2), depth + 1);
                let a_eq =
                    self.expect_beq_internal((a1, s1, var_map1), (a2, s2, var_map2), depth + 1);
                constraints_eq && f_eq && a_eq
            }
            CorePrismExpr::Free => {
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
            &CorePrismExpr::Shift(v1, i) => {
                self.expect_beq_free((v1, &s1.shift(i), var_map1), (i2, s2, var_map2), depth + 1)
            }
            &CorePrismExpr::TypeAssert(v, _t) => {
                self.expect_beq_free((v, s1, var_map1), (i2, s2, var_map2), depth + 1)
            }
            CorePrismExpr::GrammarType => {
                self.checked_values[*i2] = CorePrismExpr::GrammarType;
                self.handle_constraints(i2, s2, depth + 1)
            }
            CorePrismExpr::GrammarValue(g) => {
                self.checked_values[*i2] = CorePrismExpr::GrammarValue(g.clone());
                self.handle_constraints(i2, s2, depth + 1)
            }
        }
    }

    #[must_use]
    pub fn handle_constraints(&mut self, i2: CoreIndex, s2: &DbEnv, depth: usize) -> bool {
        let mut eq = true;

        // Check queued constraints
        if let Some(queued) = self.queued_beq_free.remove(&i2) {
            for ((s2n, mut var_map2n), (i3, s3, mut var_map3)) in queued {
                // Sanity checks
                assert_eq!(s2.len(), s2n.len());

                //TODO performance: this causes expect_beq(i3, i2) to be executed
                eq &= self.expect_beq_internal(
                    (i2, &s2n, &mut var_map2n),
                    (i3, &s3, &mut var_map3),
                    depth + 1,
                );
            }
        }

        // if let Some((s, t2)) = self.queued_tc.remove(&i2) {
        //     let t1 = self._type_check(i2, &s);
        //     eq &= self.expect_beq_internal((t1, &s, &mut HashMap::new()), (t2, &s, &mut HashMap::new()));
        // }

        eq
    }
}
