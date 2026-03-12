// use crate::lang::Expr;
// use crate::lang::env::PrismEnv;
// use crate::lang::env::EnvEntry::*;
// use crate::lang::{CoreIndex, Database};
// use crate::type_check::CheckerSessionState;
//
// impl Database {
//     pub fn is_beta_equal(&mut self, i1: CoreIndex, s1: &PrismEnv, i2: CoreIndex, s2: &PrismEnv) -> bool {
//         let mut env = CheckerSessionState::new(self);
//         env.is_beta_equal(i1, s1, i2, s2)
//     }
// }
//
// impl CheckerSessionState<'_> {
//     fn is_beta_equal(&mut self, i1: CoreIndex, s1: &PrismEnv, i2: CoreIndex, s2: &PrismEnv) -> bool {
//         // Brh and reduce i1 and i2
//         let (i1, s1) = self.db.beta_reduce_head(i1, s1);
//         let (i2, s2) = self.db.beta_reduce_head(i2, s2);
//
//         match (&self.db.exprs[*i1], &self.db.exprs[*i2]) {
//             (Expr::Type, Expr::Type) => {}
//             (&Expr::DeBruijnIndex { idx: i1 }, &Expr::DeBruijnIndex { idx: i2 }) => {
//                 let id1 = match s1[i1] {
//                     CType(id, _, _) | RType(id) => id,
//                     CSubst(..) | RSubst(..) => unreachable!(),
//                 };
//                 let id2 = match s2[i2] {
//                     CType(id, _, _) | RType(id) => id,
//                     CSubst(..) | RSubst(..) => unreachable!(),
//                 };
//                 if id1 != id2 {
//                     return false;
//                 }
//             }
//             (
//                 &Expr::FnType {
//                     arg_name: _,
//                     arg_type: a1,
//                     body: b1,
//                 },
//                 &Expr::FnType {
//                     arg_name: _,
//                     arg_type: a2,
//                     body: b2,
//                 },
//             ) => {
//                 if !self.is_beta_equal(a1, &s1, a2, &s2) {
//                     return false;
//                 }
//                 let id = self.new_tc_id();
//                 if !self.is_beta_equal(b1, &s1.cons(RType(id)), b2, &s2.cons(RType(id))) {
//                     return false;
//                 }
//             }
//             (
//                 &Expr::FnConstruct {
//                     arg_name: _,
//                     arg_type: at1,
//                     body: b1,
//                 },
//                 &Expr::FnConstruct {
//                     arg_name: _,
//                     arg_type: at2,
//                     body: b2,
//                 },
//             ) => {
//                 if !self.is_beta_equal(at1, &s1, at2, &s2) {
//                     return false;
//                 }
//                 let id = self.new_tc_id();
//                 if !self.is_beta_equal(b1, &s1.cons(RType(id)), b2, &s2.cons(RType(id))) {
//                     return false;
//                 }
//             }
//             (
//                 &Expr::FnDestruct {
//                     function: f1,
//                     arg: a1,
//                 },
//                 &Expr::FnDestruct {
//                     function: f2,
//                     arg: a2,
//                 },
//             ) => {
//                 if !self.is_beta_equal(f1, &s1, f2, &s2) {
//                     return false;
//                 }
//                 if !self.is_beta_equal(a1, &s1, a2, &s2) {
//                     return false;
//                 }
//             }
//             (Expr::Free, Expr::Free) => {}
//             _ => {
//                 return false;
//             }
//         }
//
//         true
//     }
// }
