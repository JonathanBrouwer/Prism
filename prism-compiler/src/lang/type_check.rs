use std::collections::HashMap;
use crate::lang::env::Env;
use crate::lang::env::EnvEntry::*;
use crate::lang::error::{AggregatedTypeError, TypeError};
use crate::lang::error::TypeError::IndexOutOfBound;
use crate::lang::UnionIndex;
use crate::lang::ValueOrigin::{FreeTypeFailure, FreeValueFailure, TypeOf};
use crate::lang::{PartialExpr, TcEnv};
use std::mem;

impl TcEnv {
    pub fn type_check(&mut self, root: UnionIndex) -> Result<UnionIndex, AggregatedTypeError> {
        let ti = self._type_check(root, &Env::new());

        let errors = mem::take(&mut self.errors);
        if errors.is_empty() {
            Ok(ti)
        } else {
            Err(AggregatedTypeError { errors })
        }
    }

    ///Invariant: Returned UnionIndex is valid in Env `s`
    pub(crate) fn _type_check(&mut self, i: UnionIndex, s: &Env) -> UnionIndex {
        let t = match self.values[i.0] {
            PartialExpr::Type => PartialExpr::Type,
            PartialExpr::Let(mut v, b) => {
                // Check `v`
                let err_count = self.errors.len();
                let vt = self._type_check(v, s);
                if self.errors.len() > err_count {
                    v = self.store(PartialExpr::Free, FreeValueFailure(v));
                }

                let bt = self._type_check(b, &s.cons(CSubst(v, vt)));
                PartialExpr::Let(v, bt)
            }
            PartialExpr::Var(index) => PartialExpr::Shift(
                match s.get(index) {
                    Some(&CType(_, t)) => t,
                    Some(&CSubst(_, t)) => t,
                    None => {
                        self.errors.push(IndexOutOfBound(i));
                        self.store(PartialExpr::Free, FreeTypeFailure(i))
                    }
                    _ => unreachable!(),
                },
                index + 1,
            ),
            PartialExpr::FnType(mut a, b) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, s);
                self.expect_beq_type(at, s);
                if self.errors.len() > err_count {
                    a = self.store(PartialExpr::Free, FreeValueFailure(a));
                }

                let err_count = self.errors.len();
                let bs = s.cons(CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs);
                if self.errors.len() == err_count {
                    self.expect_beq_type(bt, &bs);
                }

                PartialExpr::Type
            }
            PartialExpr::FnConstruct(mut a, b) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, s);
                self.expect_beq_type(at, s);
                if self.errors.len() > err_count {
                    a = self.store(PartialExpr::Free, FreeValueFailure(a));
                }

                let bs = s.cons(CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs);
                PartialExpr::FnType(a, bt)
            }
            PartialExpr::FnDestruct(f, mut a) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, s);
                if self.errors.len() > err_count {
                    a = self.store(PartialExpr::Free, FreeValueFailure(a));
                };

                let rt = self.store(PartialExpr::Free, TypeOf(i));

                let err_count = self.errors.len();
                let ft = self._type_check(f, s);
                if self.errors.len() == err_count {
                    self.expect_beq_fn_type(ft, at, rt, s);
                }

                PartialExpr::Let(a, rt)
            }
            PartialExpr::Free => {
                let tid = self.store(PartialExpr::Free, TypeOf(i));
                // TODO self.queued_tc.insert(i, (s.clone(), t));
                self.value_types.insert(i, tid);
                return tid;
            }
            PartialExpr::Shift(..) => unreachable!(),
        };
        let tid = self.store(t, TypeOf(i));
        self.value_types.insert(i, tid);
        tid
    }
}
