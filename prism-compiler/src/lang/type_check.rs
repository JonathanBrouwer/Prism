use crate::lang::PrismDb;
use crate::lang::ValueOrigin;
use crate::lang::env::DbEnv;
use crate::lang::env::EnvEntry::*;
use crate::lang::error::{PrismError, TypeError};
use crate::lang::{CoreIndex, CorePrismExpr};

impl PrismDb {
    pub fn type_check(&mut self, root: CoreIndex) -> CoreIndex {
        self._type_check(root, &DbEnv::default())
    }

    /// Type checkes `i` in scope `s`. Returns the type.
    /// Invariant: Returned UnionIndex is valid in Env `s`
    fn _type_check(&mut self, i: CoreIndex, env: &DbEnv) -> CoreIndex {
        // We should only type check values from the source code
        assert!(matches!(
            self.checked_origins[*i],
            ValueOrigin::SourceCode(_)
        ));

        let t = match self.checked_values[*i] {
            CorePrismExpr::Type => CorePrismExpr::Type,
            CorePrismExpr::Let(mut v, b) => {
                // Check `v`
                let err_count = self.errors.len();
                let vt = self._type_check(v, env);
                if self.errors.len() > err_count {
                    v = self.store_checked(CorePrismExpr::Free, ValueOrigin::Failure);
                }

                let bt = self._type_check(b, &env.cons(CSubst(v, vt)));
                CorePrismExpr::Let(v, bt)
            }
            CorePrismExpr::DeBruijnIndex(index) => match env.get_idx(index) {
                Some(CType(_, t) | CSubst(_, t)) => CorePrismExpr::Shift(*t, index + 1),
                Some(_) => unreachable!(),
                None => {
                    self.push_type_error(TypeError::IndexOutOfBound(i));
                    return self.store_checked(CorePrismExpr::Free, ValueOrigin::Failure);
                }
            },
            CorePrismExpr::FnType(mut a, b) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, env);
                self.expect_beq_type(at, env);
                if self.errors.len() > err_count {
                    a = self.store_checked(CorePrismExpr::Free, ValueOrigin::Failure);
                }

                let err_count = self.errors.len();
                let bs = env.cons(CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs);

                // Check if `b` typechecked without errors.
                if self.errors.len() == err_count {
                    self.expect_beq_type(bt, &bs);
                }

                CorePrismExpr::Type
            }
            CorePrismExpr::FnConstruct(b) => {
                let a = self.store_checked(CorePrismExpr::Free, ValueOrigin::FreeSub(i));
                let bs = env.cons(CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs);
                CorePrismExpr::FnType(a, bt)
            }
            CorePrismExpr::FnDestruct(f, mut a) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, env);
                if self.errors.len() > err_count {
                    a = self.store_checked(CorePrismExpr::Free, ValueOrigin::Failure);
                };

                let rt = self.store_checked(CorePrismExpr::Free, ValueOrigin::TypeOf(i));

                let err_count = self.errors.len();
                let ft = self._type_check(f, env);
                if self.errors.len() == err_count {
                    self.expect_beq_fn_type(ft, at, rt, env)
                }

                CorePrismExpr::Let(a, rt)
            }
            CorePrismExpr::TypeAssert(e, typ) => {
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
            CorePrismExpr::Free => {
                // TODO self.queued_tc.insert(i, (s.clone(), t));
                CorePrismExpr::Free
            }
            CorePrismExpr::Shift(v, shift) => {
                CorePrismExpr::Shift(self._type_check(v, &env.shift(shift)), shift)
            }
            CorePrismExpr::GrammarValue(_) => CorePrismExpr::GrammarType,
            CorePrismExpr::GrammarType => CorePrismExpr::Type,
        };
        let tid = self.store_checked(t, ValueOrigin::TypeOf(i));
        self.checked_types.insert(i, tid);
        tid
    }

    pub(super) fn push_type_error(&mut self, error: TypeError) {
        self.errors.push(PrismError::TypeError(error))
    }
}
