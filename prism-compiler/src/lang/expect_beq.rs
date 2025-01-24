use crate::lang::env::Env;
use crate::lang::env::EnvEntry::*;
use crate::lang::error::TypeError;
use crate::lang::UnionIndex;
use crate::lang::ValueOrigin::FreeSub;
use crate::lang::{PrismEnv, PrismExpr};
use std::collections::HashMap;

pub const GENERATED_NAME: &str = "GENERATED";

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    /// Expect `i1` to be equal to `i2` in `s`
    pub fn expect_beq_assert(
        &mut self,
        expr: UnionIndex,
        expr_type: UnionIndex,
        expected_type: UnionIndex,
        s: &Env<'grm>,
    ) {
        if !self.expect_beq_internal(
            (expr_type, s, &mut HashMap::new()),
            (expected_type, s, &mut HashMap::new()),
        ) {
            self.errors.push(TypeError::ExpectTypeAssert {
                expr,
                expr_type,
                expected_type,
            })
        }
    }

    /// Expect `io` to be equal to `Type`.
    pub fn expect_beq_type(&mut self, io: UnionIndex, s: &Env) {
        let (i, s) = self.beta_reduce_head(io, s.clone());
        match self.values[*i] {
            PrismExpr::Type => {}
            PrismExpr::Free => {
                self.values[*i] = PrismExpr::Type;
                if !self.handle_constraints(i, &s) {
                    self.errors.push(TypeError::ExpectType(io));
                }
            }
            _ => {
                self.errors.push(TypeError::ExpectType(io));
            }
        }
        self.toxic_values.clear();
    }

    /// Expect `f` to be a function type with argument type `i_at` both valid in `s`.
    /// `rt` should be free.
    pub fn expect_beq_fn_type(
        &mut self,
        ft: UnionIndex,
        at: UnionIndex,
        rt: UnionIndex,
        s: &Env<'grm>,
    ) {
        let (fr, sr) = self.beta_reduce_head(ft, s.clone());

        match self.values[*fr] {
            PrismExpr::FnType(_, f_at, f_rt) => {
                // Check
                if !self.expect_beq_internal(
                    (f_at, &sr, &mut HashMap::new()),
                    (at, s, &mut HashMap::new()),
                ) {
                    self.errors.push(TypeError::ExpectFnArg {
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
                    (f_rt, &sr.cons(RType(id)), &mut var_map1),
                    (rt, &s.cons(RType(id)), &mut var_map2),
                );
                assert!(is_beq_free);
            }
            PrismExpr::Free => {
                let f_at = self.store(PrismExpr::Free, FreeSub(fr));
                let f_rt = self.store(PrismExpr::Free, FreeSub(fr));
                self.values[*fr] = PrismExpr::FnType(GENERATED_NAME, f_at, f_rt);

                // TODO this won't give good errors :c
                // Figure out a way to keep the context of this constraint, maybe using tokio?
                if !self.handle_constraints(fr, &sr) {
                    self.errors.push(TypeError::ExpectFnArg {
                        function_type: ft,
                        function_arg_type: f_at,
                        arg_type: at,
                    })
                }

                let is_beq_free = self.expect_beq_free(
                    (at, s, &mut HashMap::new()),
                    (f_at, &sr, &mut HashMap::new()),
                );
                assert!(is_beq_free);
                self.toxic_values.clear();

                let mut var_map1 = HashMap::new();
                let mut var_map2 = HashMap::new();
                let id = self.new_tc_id();
                var_map1.insert(id, sr.len());
                var_map2.insert(id, s.len());
                let is_beq_free = self.expect_beq_free(
                    (f_rt, &sr.cons(RType(id)), &mut var_map1),
                    (rt, &s.cons(RType(id)), &mut var_map2),
                );
                assert!(is_beq_free);
            }
            _ => self.errors.push(TypeError::ExpectFn(ft)),
        }

        self.toxic_values.clear();
    }
}
