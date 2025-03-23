use crate::lang::CoreIndex;
use crate::lang::ValueOrigin::FreeSub;
use crate::lang::env::DbEnv;
use crate::lang::env::EnvEntry::*;
use crate::lang::error::TypeError;
use crate::lang::{CorePrismExpr, PrismEnv};
use std::collections::HashMap;

impl<'arn> PrismEnv<'arn> {
    /// Expect `i1` to be equal to `i2` in `s`
    pub fn expect_beq_assert(
        &mut self,
        expr: CoreIndex,
        expr_type: CoreIndex,
        expected_type: CoreIndex,
        s: DbEnv<'arn>,
    ) {
        if !self.expect_beq_internal(
            (expr_type, s, &mut HashMap::new()),
            (expected_type, s, &mut HashMap::new()),
            0,
        ) {
            self.push_type_error(TypeError::ExpectTypeAssert {
                expr,
                expr_type,
                expected_type,
            })
        }
    }

    /// Expect `io` to be equal to `Type`.
    pub fn expect_beq_type(&mut self, io: CoreIndex, s: DbEnv<'arn>) {
        let (i, s) = self.beta_reduce_head(io, s);
        match self.checked_values[*i] {
            CorePrismExpr::Type => {}
            CorePrismExpr::Free => {
                self.checked_values[*i] = CorePrismExpr::Type;
                if !self.handle_constraints(i, s, 0) {
                    self.push_type_error(TypeError::ExpectType(io));
                }
            }
            _ => {
                self.push_type_error(TypeError::ExpectType(io));
            }
        }
        self.toxic_values.clear();
    }

    /// Expect `f` to be a function type with argument type `i_at` both valid in `s`.
    /// `rt` should be free.
    pub fn expect_beq_fn_type(
        &mut self,
        ft: CoreIndex,
        at: CoreIndex,
        rt: CoreIndex,
        s: DbEnv<'arn>,
    ) {
        let (fr, sr) = self.beta_reduce_head(ft, s);

        match self.checked_values[*fr] {
            CorePrismExpr::FnType(f_at, f_rt) => {
                // Check
                if !self.expect_beq_internal(
                    (f_at, sr, &mut HashMap::new()),
                    (at, s, &mut HashMap::new()),
                    0,
                ) {
                    self.push_type_error(TypeError::ExpectFnArg {
                        function_type: ft,
                        function_arg_type: f_at,
                        arg_type: at,
                    })
                }
                self.toxic_values.clear();

                let mut var_map1 = HashMap::new();
                let mut var_map2 = HashMap::new();
                let id = self.new_tc_id();
                var_map1.insert(id, sr.len());
                var_map2.insert(id, s.len());
                let is_beq_free = self.expect_beq_free(
                    (f_rt, sr.cons(RType(id), self.allocs), &mut var_map1),
                    (rt, s.cons(RType(id), self.allocs), &mut var_map2),
                    0,
                );
                assert!(is_beq_free);
            }
            CorePrismExpr::Free => {
                let f_at = self.store_checked(CorePrismExpr::Free, FreeSub(fr));
                let f_rt = self.store_checked(CorePrismExpr::Free, FreeSub(fr));
                self.checked_values[*fr] = CorePrismExpr::FnType(f_at, f_rt);

                // TODO this won't give good errors :c
                // Figure out a way to keep the context of this constraint, maybe using tokio?
                if !self.handle_constraints(fr, sr, 0) {
                    self.push_type_error(TypeError::ExpectFnArg {
                        function_type: ft,
                        function_arg_type: f_at,
                        arg_type: at,
                    })
                }

                let is_beq_free = self.expect_beq_free(
                    (at, s, &mut HashMap::new()),
                    (f_at, sr, &mut HashMap::new()),
                    0,
                );
                assert!(is_beq_free);
                self.toxic_values.clear();

                let mut var_map1 = HashMap::new();
                let mut var_map2 = HashMap::new();
                let id = self.new_tc_id();
                var_map1.insert(id, sr.len());
                var_map2.insert(id, s.len());
                let is_beq_free = self.expect_beq_free(
                    (f_rt, sr.cons(RType(id), self.allocs), &mut var_map1),
                    (rt, s.cons(RType(id), self.allocs), &mut var_map2),
                    0,
                );
                assert!(is_beq_free);
            }
            _ => self.push_type_error(TypeError::ExpectFn(ft)),
        }

        self.toxic_values.clear();
    }
}
