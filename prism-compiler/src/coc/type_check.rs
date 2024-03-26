use std::mem;

use crate::coc::env::Env;
use crate::coc::env::EnvEntry::*;
use crate::coc::{PartialExpr, TcEnv, TcError};
use crate::union_find::UnionIndex;

impl TcEnv {
    pub fn type_type() -> UnionIndex {
        UnionIndex(0)
    }

    pub fn type_check(&mut self, root: UnionIndex) -> Result<UnionIndex, Vec<TcError>> {
        let ti = self.type_check_expr(root, &Env::new());
        if self.errors.is_empty() {
            Ok(ti)
        } else {
            Err(mem::take(&mut self.errors))
        }
    }

    ///Invariant: Returned UnionIndex is valid in Env `s`
    fn type_check_expr(&mut self, i: UnionIndex, s: &Env) -> UnionIndex {
        let t = match self.uf_values[self.uf.find(i).0] {
            PartialExpr::Type => PartialExpr::Type,
            PartialExpr::Let(v, b) => {
                // Check `v`
                let vt = self.type_check_expr(v, s);
                let bt = self.type_check_expr(b, &s.cons(CSubst(v, vt)));
                PartialExpr::Let(vt, bt)
            }
            PartialExpr::Var(i) => PartialExpr::Shift(
                match s[i] {
                    CType(_, t) => t,
                    CSubst(_, t) => t,
                    _ => unreachable!(),
                },
                i + 1,
            ),
            PartialExpr::FnType(a, b) => {
                let at = self.type_check_expr(a, s);
                self.expect_beq_type(at, s);
                let bs = s.cons(CType(self.new_tc_id(), a));
                let bt = self.type_check_expr(b, &bs);
                self.expect_beq_type(bt, &bs);
                PartialExpr::Type
            }
            PartialExpr::FnConstruct(a, b) => {
                let at = self.type_check_expr(a, s);
                self.expect_beq_type(at, s);
                let id = self.new_tc_id();
                let bt = self.type_check_expr(b, &s.cons(CType(id, a)));
                PartialExpr::FnType(a, bt)
            }
            PartialExpr::FnDestruct(f, a) => {
                let ft = self.type_check_expr(f, s);
                let at = self.type_check_expr(a, s);

                let rt = self.insert_union_index(PartialExpr::Free);
                let expect = self.insert_union_index(PartialExpr::FnType(at, rt));
                self.expect_beq(expect, ft, s);

                PartialExpr::Let(a, rt)
            }
            PartialExpr::Free | PartialExpr::Shift(..) => unreachable!(),
        };
        self.insert_union_index(t)
    }

    pub fn insert_union_index(&mut self, e: PartialExpr) -> UnionIndex {
        self.uf_values.push(e);
        self.uf.add()
    }
}
