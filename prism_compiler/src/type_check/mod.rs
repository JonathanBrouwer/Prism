pub(crate) mod errors;
mod expect_beq;
mod expect_beq_internal;

use crate::lang::PrismDb;
use crate::lang::ValueOrigin;
use crate::lang::env::DbEnv;
use crate::lang::env::EnvEntry::*;
use crate::lang::error::TypeError;
use crate::lang::{CoreIndex, CorePrismExpr};
use std::collections::HashMap;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct UniqueVariableId(usize);

impl UniqueVariableId {
    pub const DUMMY: UniqueVariableId = UniqueVariableId(usize::MAX);
}

type QueuedConstraint = (
    (DbEnv, HashMap<UniqueVariableId, usize>),
    (CoreIndex, DbEnv, HashMap<UniqueVariableId, usize>),
);

pub struct TypecheckPrismEnv<'a> {
    pub db: &'a mut PrismDb,

    // State during type checking
    tc_id: usize,
    queued_beq_free: HashMap<CoreIndex, Vec<QueuedConstraint>>,
    queued_tc: HashMap<CoreIndex, (DbEnv, CoreIndex)>,
}

impl<'a> TypecheckPrismEnv<'a> {
    pub fn new(db: &'a mut PrismDb) -> Self {
        Self {
            db,

            tc_id: Default::default(),
            queued_beq_free: Default::default(),
            queued_tc: Default::default(),
        }
    }

    pub fn new_tc_id(&mut self) -> UniqueVariableId {
        let id = UniqueVariableId(self.tc_id);
        self.tc_id += 1;
        id
    }

    /// Type checkes `i` in scope `s`. Returns the type.
    /// Invariant: Returned UnionIndex is valid in Env `s`
    pub fn _type_check(&mut self, i: CoreIndex, env: &DbEnv) -> CoreIndex {
        let t = match self.db.checked_values[*i] {
            CorePrismExpr::Type => CorePrismExpr::Type,
            CorePrismExpr::Let(mut v, b) => {
                // Check `v`
                let err_count = self.db.errors.len();
                let vt = self._type_check(v, env);
                if self.db.errors.len() > err_count {
                    v = self
                        .db
                        .store_checked(CorePrismExpr::Free, ValueOrigin::Failure);
                }

                let bt = self._type_check(b, &env.cons(CSubst(v, vt)));
                CorePrismExpr::Let(v, bt)
            }
            CorePrismExpr::DeBruijnIndex(index) => match env.get_idx(index) {
                Some(CType(_, t) | CSubst(_, t)) => CorePrismExpr::Shift(*t, index + 1),
                Some(_) => unreachable!(),
                None => {
                    self.push_type_error(TypeError::IndexOutOfBound(i));
                    return self
                        .db
                        .store_checked(CorePrismExpr::Free, ValueOrigin::Failure);
                }
            },
            CorePrismExpr::FnType(mut a, b) => {
                let err_count = self.db.errors.len();
                let at = self._type_check(a, env);
                self.expect_beq_type(at, env);
                if self.db.errors.len() > err_count {
                    a = self
                        .db
                        .store_checked(CorePrismExpr::Free, ValueOrigin::Failure);
                }

                let err_count = self.db.errors.len();
                let bs = env.cons(CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs);

                // Check if `b` typechecked without errors.
                if self.db.errors.len() == err_count {
                    self.expect_beq_type(bt, &bs);
                }

                CorePrismExpr::Type
            }
            CorePrismExpr::FnConstruct(b) => {
                let a = self
                    .db
                    .store_checked(CorePrismExpr::Free, ValueOrigin::FreeSub(i));
                let bs = env.cons(CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs);
                CorePrismExpr::FnType(a, bt)
            }
            CorePrismExpr::FnDestruct(f, mut a) => {
                let err_count = self.db.errors.len();
                let at = self._type_check(a, env);
                if self.db.errors.len() > err_count {
                    a = self
                        .db
                        .store_checked(CorePrismExpr::Free, ValueOrigin::Failure);
                };

                let rt = self
                    .db
                    .store_checked(CorePrismExpr::Free, ValueOrigin::TypeOf(i));

                let err_count = self.db.errors.len();
                let ft = self._type_check(f, env);
                if self.db.errors.len() == err_count {
                    self.expect_beq_fn_type(ft, at, rt, env)
                }

                CorePrismExpr::Let(a, rt)
            }
            CorePrismExpr::TypeAssert(e, typ) => {
                let err_count1 = self.db.errors.len();
                let et = self._type_check(e, env);

                let err_count2 = self.db.errors.len();
                let typt = self._type_check(typ, env);
                if self.db.errors.len() == err_count2 {
                    self.expect_beq_type(typt, env);
                }

                if self.db.errors.len() == err_count1 {
                    self.expect_beq_assert(e, et, typ, env);
                }

                return et;
            }
            CorePrismExpr::Free => {
                let tid = self
                    .db
                    .store_checked(CorePrismExpr::Free, ValueOrigin::TypeOf(i));
                self.queued_tc.insert(i, (env.clone(), tid));
                return tid;
            }
            CorePrismExpr::Shift(v, shift) => {
                CorePrismExpr::Shift(self._type_check(v, &env.shift(shift)), shift)
            }
            CorePrismExpr::GrammarValue(_) => CorePrismExpr::GrammarType,
            CorePrismExpr::GrammarType => CorePrismExpr::Type,
        };
        let tid = self.db.store_checked(t, ValueOrigin::TypeOf(i));
        self.db.checked_types.insert(i, tid);
        tid
    }

    pub(super) fn push_type_error(&mut self, error: TypeError) {
        todo!()
        // self.db.errors.push(PrismError::TypeError(error))
    }
}

impl PrismDb {
    pub fn type_check(&mut self, root: CoreIndex) -> CoreIndex {
        let mut env = TypecheckPrismEnv::new(self);
        env._type_check(root, &DbEnv::default())
    }
}
