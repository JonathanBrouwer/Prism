use std::mem;
use crate::coc::env::Env;
use crate::coc::env::EnvEntry::*;
use crate::coc::{PartialExpr, TcEnv};
use crate::coc::UnionIndex;

pub type TcError = ();

impl TcEnv {
    pub fn type_type() -> UnionIndex {
        UnionIndex(0)
    }

    pub fn type_check(&mut self, root: UnionIndex) -> Result<UnionIndex, Vec<TcError>> {
        let ti = self.type_check_expr(root, &Env::new());

        let errors = mem::take(&mut self.errors);
        if errors.is_empty() {
            Ok(ti)
        } else {
            Err(errors)
        }
    }

    ///Invariant: Returned UnionIndex is valid in Env `s`
    fn type_check_expr(&mut self, i: UnionIndex, s: &Env) -> UnionIndex {
        let t = match self.values[i.0] {
            PartialExpr::Type => PartialExpr::Type,
            PartialExpr::Let(v, b) => {
                // Check `v`
                let vt = self.type_check_expr(v, s);
                let bt = self.type_check_expr(b, &s.cons(CSubst(v, vt)));
                PartialExpr::Let(v, bt)
            }
            PartialExpr::Var(i) => PartialExpr::Shift(
                match s.get(i) {
                    Some(&CType(_, t)) => t,
                    Some(&CSubst(_, t)) => t,
                    None => {
                        self.errors.push(());
                        self.insert_union_index(PartialExpr::Free)
                    }
                    _ => unreachable!(),
                },
                i + 1,
            ),
            PartialExpr::FnType(a, b) => {
                let at = self.type_check_expr(a, s);
                self.expect_beq(at, Self::type_type(), &s);
                let bs = s.cons(CType(self.new_tc_id(), a));
                let bt = self.type_check_expr(b, &bs);
                self.expect_beq(bt, Self::type_type(), &bs);
                PartialExpr::Type
            }
            PartialExpr::FnConstruct(a, b) => {
                let at = self.type_check_expr(a, s);
                self.expect_beq(at, Self::type_type(), &s);
                let id = self.new_tc_id();
                let bt = self.type_check_expr(b, &s.cons(CType(id, a)));
                PartialExpr::FnType(a, bt)
            }
            PartialExpr::FnDestruct(f, a) => {
                let ft = self.type_check_expr(f, s);
                let at = self.type_check_expr(a, s);

                let rt = self.insert_union_index(PartialExpr::Free);
                let expect = self.insert_union_index(PartialExpr::FnType(at, rt));
                self.expect_beq(expect, ft, &s);

                PartialExpr::Let(a, rt)
            }
            PartialExpr::Free | PartialExpr::Shift(..) => unreachable!(),
        };
        self.insert_union_index(t)
    }

    pub fn insert_union_index(&mut self, e: PartialExpr) -> UnionIndex {
        self.values.push(e);
        UnionIndex(self.values.len() - 1)
    }
}
