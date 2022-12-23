// use by_address::ByAddress;
// use crate::coc::EnvEntry::{NSubst, NType};
// use crate::coc::Expr::*;
// use rpds::Vector;
//
// pub type W<T> = Box<T>;
//
// pub enum Expr {
//     Type,
//     Let(W<Self>, W<Self>),
//     Var(usize),
//     FnType(W<Self>, W<Self>),
//     FnConstruct(W<Self>, W<Self>),
//     FnDestruct(W<Self>, W<Self>),
// }
//
// #[derive(Clone)]
// pub enum EnvEntry<'a> {
//     NType(&'a Expr),
//     NSubst(SExpr<'a>),
// }
//
// pub type Env<'a> = Vector<EnvEntry<'a>>;
//
// pub type SExpr<'a> = (&'a Expr, Env<'a>);
//
// pub fn brh<'a>((eo, so): &SExpr<'a>) -> SExpr<'a> {
//     let mut args = Vec::new();
//
//     let mut e: &'a Expr = eo;
//     let mut s: Vector<EnvEntry<'a>> = so.clone();
//
//     loop {
//         match e {
//             Type => {
//                 debug_assert!(args.is_empty());
//                 return (e, s);
//             }
//             Let(v, b) => {
//                 e = b;
//                 s.push_back_mut(NSubst((v, s.clone())));
//             }
//             Var(i) => match &s[*i] {
//                 NType(_) => {
//                     return if args.len() == 0 {
//                         (e, s)
//                     } else {
//                         (eo, so.clone())
//                     }
//                 }
//                 NSubst((ne, ns)) => {
//                     e = ne;
//                     s = ns.clone();
//                 }
//             },
//             FnType(_, _) => {
//                 debug_assert!(args.is_empty());
//                 return (e, s);
//             }
//             FnConstruct(_, b) => match args.pop() {
//                 None => return (e, s.clone()),
//                 Some(arg) => {
//                     e = b;
//                     s.push_back_mut(NSubst(arg));
//                 }
//             },
//             FnDestruct(f, a) => {
//                 e = f;
//                 args.push((a, s.clone()));
//             }
//         }
//     }
// }
//
// pub fn br(e: &SExpr) -> Expr {
//     let (e, s) = brh(e);
//     match e {
//         Type => Type,
//         Let(_, _) => unreachable!(),
//         Var(i) => Var(*i),
//         FnType(a, b) => FnType(
//             W::new(br(&(a, s.clone()))),
//             W::new(br(&(b, s.push_back(NType(a))))),
//         ),
//         FnConstruct(a, b) => FnConstruct(
//             W::new(br(&(a, s.clone()))),
//             W::new(br(&(b, s.push_back(NType(a))))),
//         ),
//         FnDestruct(f, a) => FnDestruct(
//             W::new(br(&(f, s.clone()))),
//             W::new(br(&(a, s.clone()))),
//         )
//     }
// }
//
// pub fn beq(e1: &SExpr, e2: &SExpr) -> Result<(), ()> {
//     match (brh(e1), brh(e2)) {
//         ((Type, _), (Type, _)) => Ok(()),
//         ((Var(i), s1), (Var(j), s2)) if ByAddress(&s1[*i]) == ByAddress(&s2[*j]) => Ok(()),
//         // ((FnType()))
//         //TODO
//         (_, _) => Err(()),
//     }
// }
//
// pub fn tc((e, s): &SExpr) -> Result<Expr, ()> {
//     match e {
//         Type => Ok(Type),
//         Let(v, b) => {
//             let vt = tc(&(v, s.clone()))?;
//
//             Ok(vt)
//         }
//         Var(_) => todo!(),
//         FnType(_, _) => todo!(),
//         FnConstruct(_, _) => todo!(),
//         FnDestruct(_, _) => todo!(),
//     }
// }
//
// pub fn shift(e: Expr, d: isize) -> Expr {
//     todo!()
// }