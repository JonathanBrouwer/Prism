use crate::coc::env::Env;
use crate::coc::env::EnvEntry::*;
use crate::coc::{PartialExpr, TcEnv};
use crate::union_find::UnionIndex;

pub type TcError = ();

impl TcEnv {
    pub fn type_type() -> UnionIndex {
        UnionIndex(0)
    }

    pub fn type_check(&mut self, root: UnionIndex) -> Result<UnionIndex, Vec<TcError>> {
        let mut constraints = Vec::new();
        let ti = self.type_check_expr(root, &Env::new(), &mut constraints);
        
        let errors = self.solve_constraints(&constraints);
        if errors.is_empty() {
            Ok(ti)
        } else {
            Err(errors)
        }
    }

    ///Invariant: Returned UnionIndex is valid in Env `s`
    fn type_check_expr(&mut self, i: UnionIndex, s: &Env, constraints: &mut Vec<(UnionIndex, UnionIndex, Env)>) -> UnionIndex {
        let t = match self.uf_values[self.uf.find(i).0] {
            PartialExpr::Type => PartialExpr::Type,
            PartialExpr::Let(v, b) => {
                // Check `v`
                let vt = self.type_check_expr(v, s, constraints);
                let bt = self.type_check_expr(b, &s.cons(CSubst(v, vt)), constraints);
                PartialExpr::Let(v, bt)
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
                let at = self.type_check_expr(a, s, constraints);
                constraints.push((at, Self::type_type(), s.clone()));
                let bs = s.cons(CType(self.new_tc_id(), a));
                let bt = self.type_check_expr(b, &bs, constraints);
                constraints.push((bt, Self::type_type(), bs));
                PartialExpr::Type
            }
            PartialExpr::FnConstruct(a, b) => {
                let at = self.type_check_expr(a, s, constraints);
                constraints.push((at, Self::type_type(), s.clone()));
                let id = self.new_tc_id();
                let bt = self.type_check_expr(b, &s.cons(CType(id, a)), constraints);
                PartialExpr::FnType(a, bt)
            }
            PartialExpr::FnDestruct(f, a) => {
                let ft = self.type_check_expr(f, s, constraints);
                let at = self.type_check_expr(a, s, constraints);

                let rt = self.insert_union_index(PartialExpr::Free);
                let expect = self.insert_union_index(PartialExpr::FnType(at, rt));
                constraints.push((expect, ft, s.clone()));

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
    
    pub fn solve_constraints(&mut self, constraints: &Vec<(UnionIndex, UnionIndex, Env)>) -> Vec<TcError> {
        let mut errors = Vec::new();
        for (i1, i2, s) in constraints {
            self.expect_beq(*i1, *i2, s, &mut errors);
        }
        errors
    }
}
