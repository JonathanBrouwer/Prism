// use crate::lang::Expr;
// use crate::lang::env::{PrismEnv, EnvEntry};
// use crate::lang::{CoreIndex, PrismDb};
// use crate::type_check::{TypecheckPrismEnv, UniqueVariableId};
// use std::collections::HashMap;
//
// impl PrismDb {
//     pub fn simplify(&mut self, i: CoreIndex) -> CoreIndex {
//         let mut env = TypecheckPrismEnv::new(self);
//         env.simplify_inner(i, &PrismEnv::default(), &mut HashMap::new())
//     }
// }
//
// impl TypecheckPrismEnv<'_> {
//     fn simplify_inner(
//         &mut self,
//         i: CoreIndex,
//         s: &PrismEnv,
//         var_map: &mut HashMap<UniqueVariableId, usize>,
//     ) -> CoreIndex {
//         let e_new = match &self.db.exprs[*i] {
//             Expr::Type => Expr::Type,
//             &Expr::Let {
//                 name,
//                 value: v,
//                 body: b,
//             } => {
//                 let v = self.simplify_inner(v, s, var_map);
//                 let id = self.new_tc_id();
//                 var_map.insert(id, var_map.len());
//                 let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
//                 var_map.remove(&id);
//                 Expr::Let {
//                     name,
//                     value: v,
//                     body: b,
//                 }
//             }
//             &Expr::DeBruijnIndex { idx: v } => match s.get_idx(v) {
//                 Some(EnvEntry::CType(_, _, _)) | Some(EnvEntry::CSubst(_, _, _)) => unreachable!(),
//                 Some(EnvEntry::RType(id)) => Expr::DeBruijnIndex {
//                     idx: var_map.len() - var_map[id] - 1,
//                 },
//                 Some(EnvEntry::RSubst(subst, subst_env)) => {
//                     return self.simplify_inner(*subst, subst_env, var_map);
//                 }
//                 None => Expr::DeBruijnIndex { idx: v },
//             },
//             &Expr::FnType {
//                 arg_name,
//                 arg_type: a,
//                 body: b,
//             } => {
//                 let a = self.simplify_inner(a, s, var_map);
//                 let id = self.new_tc_id();
//                 var_map.insert(id, var_map.len());
//                 let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
//                 var_map.remove(&id);
//                 Expr::FnType {
//                     arg_name,
//                     arg_type: a,
//                     body: b,
//                 }
//             }
//             &Expr::FnConstruct {
//                 arg_name,
//                 arg_type,
//                 body: b,
//             } => {
//                 let arg_type = self.simplify_inner(arg_type, s, var_map);
//                 let id = self.new_tc_id();
//                 var_map.insert(id, var_map.len());
//                 let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
//                 var_map.remove(&id);
//                 Expr::FnConstruct {
//                     arg_name,
//                     arg_type,
//                     body: b,
//                 }
//             }
//             &Expr::FnDestruct {
//                 function: a,
//                 arg: b,
//             } => {
//                 let a = self.simplify_inner(a, s, var_map);
//                 let b = self.simplify_inner(b, s, var_map);
//                 Expr::FnDestruct {
//                     function: a,
//                     arg: b,
//                 }
//             }
//             Expr::Free => Expr::Free,
//             &Expr::Shift(b, i) => {
//                 return self.simplify_inner(b, &s.shift(i.min(s.len())), var_map);
//             }
//             &Expr::TypeAssert {
//                 value: e,
//                 type_hint: typ,
//             } => {
//                 let e = self.simplify_inner(e, s, var_map);
//                 let typ = self.simplify_inner(typ, s, var_map);
//                 Expr::TypeAssert {
//                     value: e,
//                     type_hint: typ,
//                 }
//             }
//         };
//         self.db.store(e_new, self.db.expr_origins[*i])
//     }
// }
