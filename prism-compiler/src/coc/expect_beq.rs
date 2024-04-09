use crate::coc::env::EnvEntry::*;
use crate::coc::env::{Env, UniqueVariableId};
use crate::coc::UnionIndex;
use crate::coc::{PartialExpr, TcEnv};
use std::collections::HashMap;

impl TcEnv {
    /// Invariant: `a` and `b` are valid in `s`
    /// Returns whether the expectation succeeded
    pub fn expect_beq(&mut self, i1: UnionIndex, i2: UnionIndex, s: &Env) {
        self.expect_beq_internal((i1, s, &mut HashMap::new()), (i2, s, &mut HashMap::new()));
        self.toxic_values.clear();
    }

    ///Invariant: `a` and `b` are valid in `s`
    fn expect_beq_internal(
        &mut self,
        (i1, s1, var_map1): (UnionIndex, &Env, &mut HashMap<UniqueVariableId, usize>),
        (i2, s2, var_map2): (UnionIndex, &Env, &mut HashMap<UniqueVariableId, usize>),
    ) {
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
                    self.errors.push(());
                }
            }
            (PartialExpr::FnType(a1, b1), PartialExpr::FnType(a2, b2)) => {
                self.expect_beq_internal((a1, &s1, var_map1), (a2, &s2, var_map2));
                let id = self.new_tc_id();
                var_map1.insert(id, s1.len());
                var_map2.insert(id, s2.len());
                self.expect_beq_internal(
                    (b1, &s1.cons(RType(id)), var_map1),
                    (b2, &s2.cons(RType(id)), var_map2),
                );
            }
            (PartialExpr::FnConstruct(a1, b1), PartialExpr::FnConstruct(a2, b2)) => {
                self.expect_beq_internal((a1, &s1, var_map1), (a2, &s2, var_map2));
                let id = self.new_tc_id();
                var_map1.insert(id, s1.len());
                var_map2.insert(id, s2.len());
                self.expect_beq_internal(
                    (b1, &s1.cons(RType(id)), var_map1),
                    (b2, &s2.cons(RType(id)), var_map2),
                );
            }
            (PartialExpr::FnDestruct(f1, a1), PartialExpr::FnDestruct(f2, a2)) => {
                self.expect_beq_internal((f1, &s1, var_map1), (f2, &s2, var_map2));
                self.expect_beq_internal((a1, &s1, var_map1), (a2, &s2, var_map2));
            }
            (_, PartialExpr::Free) => {
                self.expect_beq_free((i1, &s1, var_map1), (i2, &s2, var_map2));
            }
            (PartialExpr::Free, _) => {
                self.expect_beq_free((i2, &s2, var_map2), (i1, &s1, var_map1));
            }
            _ => {
                self.errors.push(());
            }
        }
    }

    // i2 should be free
    fn expect_beq_free(
        &mut self,
        (i1, s1, var_map1): (UnionIndex, &Env, &mut HashMap<UniqueVariableId, usize>),
        (i2, s2, var_map2): (UnionIndex, &Env, &mut HashMap<UniqueVariableId, usize>),
    ) {
        debug_assert!(matches!(self.values[i2.0], PartialExpr::Free));

        if self.toxic_values.contains(&i1) {
            self.errors.push(());
            return;
        }

        return match self.values[i1.0] {
            PartialExpr::Type => {
                self.values[i2.0] = PartialExpr::Type;
                self.handle_constraints(i2, s2);
            }
            PartialExpr::Let(v1, b1) => {
                self.expect_beq_free(
                    (b1, &s1.cons(RSubst(v1, s1.clone())), var_map1),
                    (i2, &s2, var_map2),
                );
            }
            PartialExpr::Var(v1) => {
                match &s1[v1] {
                    &CType(id, _) => {
                        // We may have shifted away part of the env that we need during this beq
                        // AFAIK this can only happen in programs that have too many frees in them
                        // We still need to check for this since otherwise we'll continue type checking based on wrong assumptions
                        let Some(v2) = (v1 + s2.len()).checked_sub(s1.len()) else {
                            self.errors.push(());
                            return;
                        };
                        let CType(id2, _) = s2[v2] else {
                            self.errors.push(());
                            return;
                        };
                        // Sanity check, after the correct value is shifted away it should not be possible for another C value to reappear
                        debug_assert_eq!(id, id2);

                        self.values[i2.0] = PartialExpr::Var(v2)
                    }
                    &CSubst(_, _) => {
                        // Same story as above, except we don't have the `id` to double check with here.
                        // The logic should still hold even without the sanity check though
                        let Some(v2) = (v1 + s2.len()).checked_sub(s1.len()) else {
                            self.errors.push(());
                            return;
                        };
                        let CSubst(_, _) = s2[v2] else {
                            self.errors.push(());
                            return;
                        };

                        self.values[i2.0] = PartialExpr::Var(v2);
                    }
                    &RType(id) => {
                        // If `id` still exists in s2, we will find it here
                        let Some(v2) = s2.len().checked_sub(var_map2[&id] + 1) else {
                            self.errors.push(());
                            return;
                        };
                        // Check if it still exists, if not we shifted it out and another entry came in the place for it
                        let RType(id2) = s2[v2] else {
                            self.errors.push(());
                            panic!("I think this can happen but I want an example for it");
                            // TODO return;
                        };
                        if id != id2 {
                            self.errors.push(());
                            panic!("I think this can happen but I want an example for it");
                            // TODO return;
                        }

                        self.values[i2.0] = PartialExpr::Var(v2);
                    }
                    RSubst(i1, s1) => {
                        self.expect_beq_free((*i1, &s1, var_map1), (i2, s2, var_map2));
                    }
                }
                self.handle_constraints(i2, s2);
            }
            PartialExpr::FnType(a1, b1) => {
                let a2 = self.store(PartialExpr::Free);
                let b2 = self.store(PartialExpr::Free);
                self.values[i2.0] = PartialExpr::FnType(a2, b2);

                self.handle_constraints(i2, s2);

                self.toxic_values.insert(i2);
                self.expect_beq_free((a1, &s1, var_map1), (a2, s2, var_map2));
                let id = self.new_tc_id();
                var_map1.insert(id, s1.len());
                var_map2.insert(id, s2.len());
                self.expect_beq_free(
                    (b1, &s1.cons(RType(id)), var_map1),
                    (b2, &s2.cons(RType(id)), var_map2),
                );
            }
            PartialExpr::FnConstruct(a1, b1) => {
                // let a2 = self.store(PartialExpr::Free);
                // let b2 = self.store(PartialExpr::Free);
                // self.values[i2.0] = PartialExpr::FnConstruct(a2, b2);
                //
                // self.toxic_values.insert(i2);
                // self.expect_beq_free((a1, &s1, var_map1), (a2, &s2, var_map2));
                //
                // let id = self.new_tc_id();
                // var_map1.insert(id, s1.len());
                // var_map2.insert(id, s2.len());
                // self.expect_beq_free(
                //     (b1, &s1.cons(RType(id)), var_map1),
                //     (b2, &s2.cons(RType(id)), var_map2),
                // );
                //
                // self.handle_constraints(i2, s2);
                // self.toxic_values.remove(&i2);

                //TODO
                self.errors.push(());
                return;
            }
            PartialExpr::FnDestruct(_, _) => {
                //         let f2 = self.store(PartialExpr::Free);
                //         let a2 = self.store(PartialExpr::Free);
                //         self.values[i2.0] = TOXIC;
                //
                //         self.expect_beq_free((f1, &s1, var_map1), (f2, &s2, var_map2));
                //         self.expect_beq_free((a1, &s1, var_map1), (a2, &s2, var_map2));
                //
                //         self.values[i2.0] = PartialExpr::FnDestruct(f2, a2);
                //TODO
                self.errors.push(());
                return;

                self.handle_constraints(i2, s2);
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
            }
            PartialExpr::Shift(v1, i) => {
                self.expect_beq_free((v1, &s1.shift(i), var_map1), (i2, &s2, var_map2));
            }
        }

        // if let Some((s, t2)) = self.queued_tc.remove(&i2) {
        //     let t1 = self._type_check(i2, &s);
        //     self.expect_beq(t1, t2, &s);
        // }
    }

    fn handle_constraints(&mut self, i2: UnionIndex, s2: &Env) {
        // Check queued constraints
        if let Some(queued) = self.queued_beq_free.remove(&i2) {
            for ((s2n, mut var_map2n), (i3, s3, mut var_map3)) in queued {
                // Sanity checks
                debug_assert_eq!(s2.len(), s2n.len());

                //TODO performance: this causes expect_beq(i3, i2) to be executed
                self.expect_beq_internal((i2, &s2n, &mut var_map2n), (i3, &s3, &mut var_map3));
            }
        }
    }
}
