use crate::coc::env::{Env, EnvEntry, SExpr};
use crate::coc::Expr;
use crate::coc::type_check::TcEnv;
use crate::union_find::UnionIndex;

// impl<'arn> TcEnv<'arn> {
//     pub fn brh(&mut self, (eo, so): SExpr<'arn>) -> SExpr<'arn> {
//         // Used if we need to insert types during beta reduction that we don't know
//         const PLACE_HOLDERTYPE: UnionIndex = UnionIndex(usize::MAX);
//
//         let mut args = Vec::new();
//
//         let mut e: &'arn Expr<'arn> = eo;
//         let mut s: Env<'arn> = so.clone();
//
//         loop {
//             match &e {
//                 Expr::Type => {
//                     debug_assert!(args.is_empty());
//                     return (e, s);
//                 }
//                 Expr::Let(v, b) => {
//                     e = b;
//                     s = s.cons(EnvEntry::NSubst(PLACE_HOLDERTYPE, (v, s.clone())));
//                 }
//                 Expr::Var(i) => match &s[*i] {
//                     EnvEntry::NType(_) => {
//                         return if args.len() == 0 {
//                             (e, s)
//                         } else {
//                             (eo, so.clone())
//                         }
//                     }
//                     EnvEntry::NSubst(_, (ne, ns)) => {
//                         e = ne;
//                         s = ns.clone();
//                     }
//                 },
//                 Expr::FnType(_, _) => {
//                     debug_assert!(args.is_empty());
//                     return (e, s);
//                 }
//                 Expr::FnConstruct(_, b) => match args.pop() {
//                     None => return (e, s.clone()),
//                     Some(arg) => {
//                         e = b;
//                         s = s.cons(EnvEntry::NSubst(PLACE_HOLDERTYPE, arg));
//                     }
//                 },
//                 Expr::FnDestruct(f, a) => {
//                     e = f;
//                     args.push((a, s.clone()));
//                 }
//             }
//         }
//     }
// }
//
