use crate::lang::env::EnvEntry::*;
use crate::lang::env::{Env};
use crate::lang::UnionIndex;
use crate::lang::{PartialExpr, TcEnv};
use std::collections::HashMap;
use crate::lang::error::TypeError;

impl TcEnv {
    /// Invariant: `i1` and `i2` are valid in `s`
    /// Returns whether the expectation succeeded
    pub fn expect_beq(&mut self, i1: UnionIndex, i2: UnionIndex, s: &Env) {
        if !self.expect_beq_internal((i1, s, &mut HashMap::new()), (i2, s, &mut HashMap::new())) {
            //TODO fix this
            self.errors.push(TypeError::InfiniteType(i1));
        }
        self.toxic_values.clear();
    }

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
    }

    /// Expect `i_o` to be a function type with argument type `i_a` both valid in `s`, returns the return value of the function.
    /// Note: Return value needs to be shifted with `1` relative to `s`.
    pub fn expect_beq_fn_type(&mut self, i_fo: UnionIndex, i_a: UnionIndex, s: &Env) -> UnionIndex {
        // let (i_f, s_f) = self.beta_reduce_head(i_fo, s.clone());
        // match self.values[i_f.0] {
        //     PartialExpr::FnType(a, b) => {
        //         let mut var_map1 = HashMap::new();
        //         let mut var_map2 = HashMap::new();
        //         
        //         self.expect_beq_internal((i_a, s, &mut var_map1), (a, &s_f, &mut var_map2));
        // 
        //         let id = self.new_tc_id();
        //         var_map1.insert(id, s1.len());
        //         var_map2.insert(id, s2.len());
        //         self.expect_beq_internal(
        //             (b1, &s1.cons(RType(id)), var_map1),
        //             (b2, &s2.cons(RType(id)), var_map2),
        //         );
        //         
        //         todo!()
        //     }
        //     PartialExpr::Free => {
        //         self.values[i.0] = PartialExpr::Type;
        //         self.handle_constraints(i, &s);
        // 
        //         todo!()
        //     }
        //     _ => {
        //         self.errors.push(TypeError::ExpectType(io));
        // 
        //         todo!()
        //     }
        // }
        todo!()
    }
}
