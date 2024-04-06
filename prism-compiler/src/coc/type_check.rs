use crate::coc::env::Env;
use crate::coc::env::EnvEntry::*;
use crate::coc::UnionIndex;
use crate::coc::{PartialExpr, TcEnv};
use std::mem;

pub type TcError = ();

impl TcEnv {
    pub fn type_check(&mut self, root: UnionIndex) -> Result<UnionIndex, Vec<TcError>> {
        let ti = self._type_check(root, &Env::new());

        let errors = mem::take(&mut self.errors);
        if errors.is_empty() {
            Ok(ti)
        } else {
            Err(errors)
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
                    v = self.store(PartialExpr::Free);
                }

                let bt = self._type_check(b, &s.cons(CSubst(v, vt)));
                PartialExpr::Let(v, bt)
            }
            PartialExpr::Var(i) => PartialExpr::Shift(
                match s.get(i) {
                    Some(&CType(_, t)) => t,
                    Some(&CSubst(_, t)) => t,
                    None => {
                        self.errors.push(());
                        self.store(PartialExpr::Free)
                    }
                    _ => unreachable!(),
                },
                i + 1,
            ),
            PartialExpr::FnType(mut a, b) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, s);
                let at_expect = self.store(PartialExpr::Type);
                self.expect_beq(at, at_expect, &s);
                if self.errors.len() > err_count {
                    a = self.store(PartialExpr::Free);
                }

                let bs = s.cons(CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs);
                let bt_expect = self.store(PartialExpr::Type);
                self.expect_beq(bt, bt_expect, &bs);

                PartialExpr::Type
            }
            PartialExpr::FnConstruct(mut a, b) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, s);
                let at_expect = self.store(PartialExpr::Type);
                self.expect_beq(at, at_expect, &s);
                if self.errors.len() > err_count {
                    a = self.store(PartialExpr::Free);
                }

                let bs = s.cons(CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs);
                PartialExpr::FnType(a, bt)
            }
            PartialExpr::FnDestruct(f, mut a) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, s);
                if self.errors.len() > err_count {
                    a = self.store(PartialExpr::Free);
                };

                let rt = self.store(PartialExpr::Free);

                let err_count = self.errors.len();
                let ft = self._type_check(f, s);
                if self.errors.len() == err_count {
                    let expect = self.store(PartialExpr::FnType(at, rt));
                    self.expect_beq(expect, ft, &s);
                }

                PartialExpr::Let(a, rt)
            }
            PartialExpr::Free => {
                let t = self.store(PartialExpr::Free);
                // TODO self.queued_tc.insert(i, (s.clone(), t));
                return t;
            }
            PartialExpr::Shift(..) => unreachable!(),
        };
        self.store(t)
    }

    pub fn store(&mut self, e: PartialExpr) -> UnionIndex {
        self.values.push(e);
        UnionIndex(self.values.len() - 1)
    }
}
