use crate::lang::PrismEnv;
use crate::lang::ValueOrigin;
use crate::lang::env::DbEnv;
use crate::lang::env::EnvEntry::*;
use crate::lang::error::{AggregatedTypeError, TypeError};
use crate::lang::{CheckedIndex, CheckedPrismExpr};
use std::mem;

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn type_check(&mut self, root: CheckedIndex) -> Result<CheckedIndex, AggregatedTypeError> {
        let ti = self._type_check(root, DbEnv::default());

        let errors = mem::take(&mut self.errors);
        if errors.is_empty() {
            Ok(ti)
        } else {
            Err(AggregatedTypeError { errors })
        }
    }

    /// Type checkes `i` in scope `s`. Returns the type.
    /// Invariant: Returned UnionIndex is valid in Env `s`
    fn _type_check(&mut self, i: CheckedIndex, env: DbEnv<'arn>) -> CheckedIndex {
        // We should only type check values from the source code
        assert!(matches!(
            self.checked_origins[*i],
            ValueOrigin::SourceCode(_)
        ));

        let t = match self.checked_values[*i] {
            CheckedPrismExpr::Type => CheckedPrismExpr::Type,
            CheckedPrismExpr::Let(mut v, b) => {
                // Check `v`
                let err_count = self.errors.len();
                let vt = self._type_check(v, env);
                if self.errors.len() > err_count {
                    v = self.store_checked(CheckedPrismExpr::Free, ValueOrigin::Failure);
                }

                let bt = self._type_check(b, env.cons(CSubst(v, vt), self.allocs));
                CheckedPrismExpr::Let(v, bt)
            }
            CheckedPrismExpr::DeBruijnIndex(index) => match env.get_idx(index) {
                Some(CType(_, t) | CSubst(_, t)) => CheckedPrismExpr::Shift(t, index + 1),
                Some(_) => unreachable!(),
                None => {
                    self.errors.push(TypeError::IndexOutOfBound(i));
                    return self.store_checked(CheckedPrismExpr::Free, ValueOrigin::Failure);
                }
            },
            CheckedPrismExpr::FnType(mut a, b) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, env);
                self.expect_beq_type(at, env);
                if self.errors.len() > err_count {
                    a = self.store_checked(CheckedPrismExpr::Free, ValueOrigin::Failure);
                }

                let err_count = self.errors.len();
                let bs = env.cons(CType(self.new_tc_id(), a), self.allocs);
                let bt = self._type_check(b, bs);

                // Check if `b` typechecked without errors.
                if self.errors.len() == err_count {
                    self.expect_beq_type(bt, bs);
                }

                CheckedPrismExpr::Type
            }
            CheckedPrismExpr::FnConstruct(b) => {
                let a = self.store_checked(CheckedPrismExpr::Free, ValueOrigin::FreeSub(i));
                let bs = env.cons(CType(self.new_tc_id(), a), self.allocs);
                let bt = self._type_check(b, bs);
                CheckedPrismExpr::FnType(a, bt)
            }
            CheckedPrismExpr::FnDestruct(f, mut a) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, env);
                if self.errors.len() > err_count {
                    a = self.store_checked(CheckedPrismExpr::Free, ValueOrigin::Failure);
                };

                let rt = self.store_checked(CheckedPrismExpr::Free, ValueOrigin::TypeOf(i));

                let err_count = self.errors.len();
                let ft = self._type_check(f, env);
                if self.errors.len() == err_count {
                    self.expect_beq_fn_type(ft, at, rt, env)
                }

                CheckedPrismExpr::Let(a, rt)
            }
            CheckedPrismExpr::TypeAssert(e, typ) => {
                let err_count1 = self.errors.len();
                let et = self._type_check(e, env);

                let err_count2 = self.errors.len();
                let typt = self._type_check(typ, env);
                if self.errors.len() == err_count2 {
                    self.expect_beq_type(typt, env);
                }

                if self.errors.len() == err_count1 {
                    self.expect_beq_assert(e, et, typ, env);
                }

                return et;
            }
            CheckedPrismExpr::Free => {
                // TODO self.queued_tc.insert(i, (s.clone(), t));
                CheckedPrismExpr::Free
            }
            CheckedPrismExpr::Shift(v, shift) => {
                CheckedPrismExpr::Shift(self._type_check(v, env.shift(shift)), shift)
            }
            CheckedPrismExpr::GrammarValue(_, _) => CheckedPrismExpr::GrammarType,
            CheckedPrismExpr::GrammarType => CheckedPrismExpr::Type,
        };
        let tid = self.store_checked(t, ValueOrigin::TypeOf(i));
        self.checked_types.insert(i, tid);
        tid
    }
}
