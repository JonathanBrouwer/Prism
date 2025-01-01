use crate::lang::env::Env;
use crate::lang::env::EnvEntry::*;
use crate::lang::error::{AggregatedTypeError, TypeError};
use crate::lang::expect_beq::GENERATED_NAME;
use crate::lang::UnionIndex;
use crate::lang::ValueOrigin;
use crate::lang::{PartialExpr, TcEnv};
use std::mem;

impl<'grm> TcEnv<'grm> {
    pub fn type_check(&mut self, root: UnionIndex) -> Result<UnionIndex, AggregatedTypeError> {
        let ti = self._type_check(root, &Env::new());

        let errors = mem::take(&mut self.errors);
        if errors.is_empty() {
            Ok(ti)
        } else {
            Err(AggregatedTypeError { errors })
        }
    }

    /// Type checkes `i` in scope `s`. Returns the type.
    /// Invariant: Returned UnionIndex is valid in Env `s`
    fn _type_check(&mut self, i: UnionIndex, s: &Env<'grm>) -> UnionIndex {
        // We should only type check values from the source code
        assert!(matches!(self.value_origins[*i], ValueOrigin::SourceCode(_)));

        let t = match self.values[*i] {
            PartialExpr::Type => PartialExpr::Type,
            PartialExpr::Let(n, mut v, b) => {
                // Check `v`
                let err_count = self.errors.len();
                let vt = self._type_check(v, s);
                if self.errors.len() > err_count {
                    v = self.store(PartialExpr::Free, ValueOrigin::Failure);
                }

                let bt = self._type_check(b, &s.cons(CSubst(v, vt, n)));
                PartialExpr::Let(n, v, bt)
            }
            PartialExpr::DeBruijnIndex(index) => match s.get(index) {
                Some(&CType(_, t, _) | &CSubst(_, t, _)) => PartialExpr::Shift(t, index + 1),
                Some(_) => unreachable!(),
                None => {
                    self.errors.push(TypeError::IndexOutOfBound(i));
                    return self.store(PartialExpr::Free, ValueOrigin::Failure);
                }
            },
            PartialExpr::FnType(n, mut a, b) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, s);
                self.expect_beq_type(at, s);
                if self.errors.len() > err_count {
                    a = self.store(PartialExpr::Free, ValueOrigin::Failure);
                }

                let err_count = self.errors.len();
                let bs = s.cons(CType(self.new_tc_id(), a, n));
                let bt = self._type_check(b, &bs);

                // Check if `b` typechecked without errors.
                if self.errors.len() == err_count {
                    self.expect_beq_type(bt, &bs);
                }

                PartialExpr::Type
            }
            PartialExpr::FnConstruct(n, b) => {
                let a = self.store(PartialExpr::Free, ValueOrigin::FreeSub(i));
                let bs = s.cons(CType(self.new_tc_id(), a, n));
                let bt = self._type_check(b, &bs);
                PartialExpr::FnType(n, a, bt)
            }
            PartialExpr::FnDestruct(f, mut a) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, s);
                if self.errors.len() > err_count {
                    a = self.store(PartialExpr::Free, ValueOrigin::Failure);
                };

                let rt = self.store(PartialExpr::Free, ValueOrigin::TypeOf(i));

                let err_count = self.errors.len();
                let ft = self._type_check(f, s);
                if self.errors.len() == err_count {
                    self.expect_beq_fn_type(ft, at, rt, s)
                }

                PartialExpr::Let(GENERATED_NAME, a, rt)
            }
            PartialExpr::Free => {
                // TODO self.queued_tc.insert(i, (s.clone(), t));
                PartialExpr::Free
            }
            PartialExpr::Shift(v, shift) => {
                PartialExpr::Shift(self._type_check(v, &s.shift(shift)), shift)
            }
            PartialExpr::TypeAssert(e, typ) => {
                let err_count1 = self.errors.len();
                let et = self._type_check(e, s);

                let err_count2 = self.errors.len();
                let typt = self._type_check(typ, s);
                if self.errors.len() == err_count2 {
                    self.expect_beq_type(typt, s);
                }

                if self.errors.len() == err_count1 {
                    self.expect_beq_assert(e, et, typ, s);
                }

                return et;
            }
            PartialExpr::Name(_) => unimplemented!(),
        };
        let tid = self.store(t, ValueOrigin::TypeOf(i));
        self.value_types.insert(i, tid);
        tid
    }
}
