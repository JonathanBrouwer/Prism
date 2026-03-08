mod errors;
mod expect_beq;
mod expect_beq_internal;

use crate::lang::PrismDb;
use crate::lang::ValueOrigin;
use crate::lang::env::DbEnv;
use crate::lang::env::EnvEntry::*;
use crate::lang::{CoreIndex, Expr};
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
        let t = match self.db.exprs[*i] {
            Expr::Type => Expr::Type,
            Expr::Let {
                name,
                value: mut v,
                body: b,
            } => {
                // Check `v`
                let recovery_token = self.db.recovery_point();
                let vt = self._type_check(v, env);
                if let Some(err) = self.db.try_recover(recovery_token) {
                    v = self.db.store(Expr::Free, ValueOrigin::Failure(err));
                }

                let bt = self._type_check(b, &env.cons(CSubst(v, vt)));
                Expr::Let {
                    name,
                    value: v,
                    body: bt,
                }
            }
            Expr::DeBruijnIndex { idx: index } => match env.get_idx(index) {
                Some(CType(_, t) | CSubst(_, t)) => Expr::Shift(*t, index + 1),
                Some(_) => unreachable!(),
                None => {
                    unreachable!("Index out of bound");
                    // self.push_type_error(TypeError::IndexOutOfBound(i));
                    // return self
                    //     .db
                    //     .store_checked(CorePrismExpr::Free, ValueOrigin::Failure);
                }
            },
            Expr::FnType {
                arg_name: _,
                arg_type: mut a,
                body: b,
            } => {
                let recovery_token = self.db.recovery_point();
                let at = self._type_check(a, env);
                self.expect_beq_type(at, env);
                if let Some(err) = self.db.try_recover(recovery_token) {
                    a = self.db.store(Expr::Free, ValueOrigin::Failure(err));
                }

                let err_count = self.db.diags.len();
                let bs = env.cons(CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs);

                // Check if `b` typechecked without errors.
                if self.db.diags.len() == err_count {
                    self.expect_beq_type(bt, &bs);
                }

                Expr::Type
            }
            Expr::FnConstruct { arg_name, body: b } => {
                let a = self.db.store(Expr::Free, ValueOrigin::FreeSub(i));
                let bs = env.cons(CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs);
                Expr::FnType {
                    arg_name,
                    arg_type: a,
                    body: bt,
                }
            }
            Expr::FnDestruct {
                function: f,
                arg: mut a,
            } => {
                let recovery_token = self.db.recovery_point();
                let at = self._type_check(a, env);
                if let Some(err) = self.db.try_recover(recovery_token) {
                    a = self.db.store(Expr::Free, ValueOrigin::Failure(err));
                }

                let rt = self.db.store(Expr::Free, ValueOrigin::TypeOf(i));

                let err_count = self.db.diags.len();
                let ft = self._type_check(f, env);
                if self.db.diags.len() == err_count {
                    self.expect_beq_fn_type(ft, at, rt, env)
                }

                Expr::Let {
                    name: None,
                    value: a,
                    body: rt,
                }
            }
            Expr::TypeAssert {
                value: e,
                type_hint: typ,
            } => {
                let err_count1 = self.db.diags.len();
                let et = self._type_check(e, env);

                let err_count2 = self.db.diags.len();
                let typt = self._type_check(typ, env);
                if self.db.diags.len() == err_count2 {
                    self.expect_beq_type(typt, env);
                }

                if self.db.diags.len() == err_count1 {
                    self.expect_beq_assert(e, et, typ, env);
                }

                return et;
            }
            Expr::Free => {
                let tid = self.db.store(Expr::Free, ValueOrigin::TypeOf(i));
                self.queued_tc.insert(i, (env.clone(), tid));
                return tid;
            }
            Expr::Shift(v, shift) => Expr::Shift(self._type_check(v, &env.shift(shift)), shift),
        };
        let tid = self.db.store(t, ValueOrigin::TypeOf(i));
        self.db.checked_types.insert(i, tid);
        tid
    }
}

impl PrismDb {
    pub fn type_check(&mut self, root: CoreIndex) -> CoreIndex {
        let mut env = TypecheckPrismEnv::new(self);
        env._type_check(root, &DbEnv::default())
    }
}
