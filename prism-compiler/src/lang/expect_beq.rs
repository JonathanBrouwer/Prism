use crate::lang::env::Env;
use crate::lang::env::EnvEntry::*;
use crate::lang::error::TypeError;
use crate::lang::UnionIndex;
use crate::lang::ValueOrigin::{FreeSub, TypeOf};
use crate::lang::{PartialExpr, TcEnv};
use std::collections::HashMap;

impl TcEnv {
    /// Expect `io` to be equal to `Type`.
    pub fn expect_beq_type(&mut self, io: UnionIndex, s: &Env) {
        let (i, s) = self.beta_reduce_head(io, s.clone());
        match self.values[i.0] {
            PartialExpr::Type => {}
            PartialExpr::Free => {
                self.values[i.0] = PartialExpr::Type;
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
    pub fn expect_beq_fn_type(&mut self, ft: UnionIndex, at: UnionIndex, rt: UnionIndex, s: &Env) {
        let (fr, sr) = self.beta_reduce_head(ft, s.clone());
        let mut var_map1 = HashMap::new();
        let mut var_map2 = HashMap::new();

        match self.values[fr.0] {
            PartialExpr::FnType(f_at, f_rt) => {
                if !self.expect_beq_internal((f_at, &sr, &mut var_map1), (at, s, &mut var_map2)) {
                    self.errors.push(TypeError::ExpectFnArg {
                        function_type: ft,
                        function_arg_type: f_at,
                        arg_type: at,
                    })
                }


                //TODO rt is unused here
                //do we need to make a new free far on tc:86?

                // let id = self.new_tc_id();
                // var_map1.insert(id, sr.len());
                // var_map2.insert(id, s.len());
                // debug_assert!(self.expect_beq_free(
                //     (f_rt, &sr.cons(RType(id)), &mut var_map1),
                //     (rt, &s.cons(RType(id)), &mut var_map2),
                // ));

                /////// TMP CODE
                // let expect = self.store(PartialExpr::FnType(at, rt), TypeOf(ft)); //TODO ft is wrong here
                // if !self.expect_beq_internal((expect, s, &mut HashMap::new()), (ft, s, &mut HashMap::new())) {
                //     self.errors.push(TypeError::ExpectFn(ft))
                // }
                //////
            }
            PartialExpr::Free => {
                /////// TMP CODE
                let expect = self.store(PartialExpr::FnType(at, rt), TypeOf(ft)); //TODO ft is wrong here
                if !self.expect_beq_internal((expect, s, &mut HashMap::new()), (ft, s, &mut HashMap::new())) {
                    self.errors.push(TypeError::ExpectFn(ft))
                }
                //////
            }
            _ => self.errors.push(TypeError::ExpectFn(ft)),
        }

        self.toxic_values.clear();


        // match self.values[fr.0] {
        //     PartialExpr::FnType(f_at, f_rt) => {
        //         if !self.expect_beq_internal((f_at, &sr, &mut var_map1), (at, s, &mut var_map2)) {
        //             self.errors.push(TypeError::ExpectFnArg {
        //                 function_type: ft,
        //                 function_arg_type: f_at,
        //                 arg_type: at,
        //             })
        //         }
        //
        //         let id = self.new_tc_id();
        //         var_map1.insert(id, sr.len());
        //         var_map2.insert(id, s.len());
        //         debug_assert!(self.expect_beq_free(
        //             (f_rt, &sr.cons(RType(id)), &mut var_map1),
        //             (rt, &s.cons(RType(id)), &mut var_map2),
        //         ));
        //     }
        //     PartialExpr::Free => {
        //         let f_at = self.store(PartialExpr::Free, FreeSub(fr));
        //         let f_rt = self.store(PartialExpr::Free, FreeSub(fr));
        //         self.values[fr.0] = PartialExpr::FnType(f_at, f_rt);
        //
        //         // TODO this won't give good errors :c
        //         // Figure out a way to keep the context of this constraint, maybe using tokio?
        //         if !self.handle_constraints(fr, &sr) {
        //             self.errors.push(TypeError::ExpectFnArg {
        //                 function_type: ft,
        //                 function_arg_type: f_at,
        //                 arg_type: at,
        //             })
        //         }
        //
        //         self.toxic_values.insert(fr);
        //         let id = self.new_tc_id();
        //         var_map1.insert(id, sr.len());
        //         var_map2.insert(id, s.len());
        //
        //         debug_assert!(self.expect_beq_free(
        //             (f_rt, &sr.cons(RType(id)), &mut var_map1),
        //             (rt, &s.cons(RType(id)), &mut var_map2),
        //         ));
        //     }
        //     _ => self.errors.push(TypeError::ExpectFn(ft)),
        // }
        //
        // self.toxic_values.clear();
    }
}
