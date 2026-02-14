use crate::lang::CoreIndex;
use crate::lang::CorePrismExpr;
use crate::lang::ValueOrigin::FreeSub;
use crate::lang::env::DbEnv;
use crate::lang::env::EnvEntry::*;
use crate::type_check::TypecheckPrismEnv;
use crate::type_check::errors::{ExpectedFn, ExpectedFnArg, ExpectedType, FailedTypeAssert};
use std::collections::HashMap;

impl<'a> TypecheckPrismEnv<'a> {
    /// Expect `i1` to be equal to `i2` in `s`
    pub fn expect_beq_assert(
        &mut self,
        expr: CoreIndex,
        expr_type: CoreIndex,
        expected_type: CoreIndex,
        s: &DbEnv,
    ) {
        if !self.expect_beq_internal(
            (expr_type, s, &mut HashMap::new()),
            (expected_type, s, &mut HashMap::new()),
            0,
        ) {
            self.db.push_error(FailedTypeAssert {
                expr,
                expr_type,
                expected_type,
            });
        }
    }

    /// Expect `io` to be equal to `Type`.
    pub fn expect_beq_type(&mut self, io: CoreIndex, s: &DbEnv) {
        let (i, s) = self.db.beta_reduce_head(io, s);
        match self.db.values[*i] {
            CorePrismExpr::Type => {}
            CorePrismExpr::Free => {
                self.db.values[*i] = CorePrismExpr::Type;
                if !self.handle_constraints(i, &s, 0) {
                    self.db.push_error(ExpectedType { index: io });
                }
            }
            _ => {
                self.db.push_error(ExpectedType { index: io });
            }
        }
    }

    /// Expect `f` to be a function type with argument type `i_at` both valid in `s`.
    /// `rt` should be free.
    pub fn expect_beq_fn_type(&mut self, ft: CoreIndex, at: CoreIndex, rt: CoreIndex, s: &DbEnv) {
        let (fr, sr) = self.db.beta_reduce_head(ft, s);

        match self.db.values[*fr] {
            CorePrismExpr::FnType(f_at, f_rt) => {
                // Check
                if !self.expect_beq_internal(
                    (f_at, &sr, &mut HashMap::new()),
                    (at, s, &mut HashMap::new()),
                    0,
                ) {
                    self.db.push_error(ExpectedFnArg {
                        function_type: ft,
                        function_arg_type: f_at,
                        arg_type: at,
                    });
                }

                let mut var_map1 = HashMap::new();
                let mut var_map2 = HashMap::new();
                let id = self.new_tc_id();
                var_map1.insert(id, sr.len());
                var_map2.insert(id, s.len());
                let is_beq_free = self.expect_beq_free(
                    (f_rt, &sr.cons(RType(id)), &mut var_map1),
                    (rt, &s.cons(RType(id)), &mut var_map2),
                    0,
                );
                assert!(is_beq_free);
            }
            CorePrismExpr::Free => {
                let f_at = self.db.store(CorePrismExpr::Free, FreeSub(fr));
                let f_rt = self.db.store(CorePrismExpr::Free, FreeSub(fr));
                self.db.values[*fr] = CorePrismExpr::FnType(f_at, f_rt);

                // TODO this won't give good errors :c
                // Figure out a way to keep the context of this constraint, maybe using tokio?
                if !self.handle_constraints(fr, &sr, 0) {
                    self.db.push_error(ExpectedFnArg {
                        function_type: ft,
                        function_arg_type: f_at,
                        arg_type: at,
                    });
                }

                let is_beq_free = self.expect_beq_free(
                    (at, s, &mut HashMap::new()),
                    (f_at, &sr, &mut HashMap::new()),
                    0,
                );
                assert!(is_beq_free);

                let mut var_map1 = HashMap::new();
                let mut var_map2 = HashMap::new();
                let id = self.new_tc_id();
                var_map1.insert(id, sr.len());
                var_map2.insert(id, s.len());
                let is_beq_free = self.expect_beq_free(
                    (f_rt, &sr.cons(RType(id)), &mut var_map1),
                    (rt, &s.cons(RType(id)), &mut var_map2),
                    0,
                );
                assert!(is_beq_free);
            }
            _ => {
                self.db.push_error(ExpectedFn { index: ft });
            }
        }
    }
}
