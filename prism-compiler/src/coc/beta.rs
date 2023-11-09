use crate::coc::env::{Env, EnvEntry, SExpr};
use crate::coc::Expr;

pub fn brh<'arn>((eo, so): SExpr<'arn>) -> SExpr<'arn> {
    let mut args = Vec::new();

    let mut e: &'arn Expr<'arn> = eo;
    let mut s: Env<'arn> = so.clone();

    loop {
        match &e {
            Expr::Type => {
                debug_assert!(args.is_empty());
                return (e, s);
            }
            Expr::Let(v, b) => {
                e = b;
                s = s.cons(EnvEntry::NSubst((v, s.clone())));
            }
            Expr::Var(i) => match &s[*i] {
                EnvEntry::NType(_) => {
                    return if args.len() == 0 {
                        (e, s)
                    } else {
                        (eo, so.clone())
                    }
                }
                EnvEntry::NSubst((ne, ns)) => {
                    e = ne;
                    s = ns.clone();
                }
            },
            Expr::FnType(_, _) => {
                debug_assert!(args.is_empty());
                return (e, s);
            }
            Expr::FnConstruct(_, b) => match args.pop() {
                None => return (e, s.clone()),
                Some(arg) => {
                    e = b;
                    s = s.cons(EnvEntry::NSubst(arg));
                }
            },
            Expr::FnDestruct(f, a) => {
                e = f;
                args.push((a, s.clone()));
            }
        }
    }
}








// use crate::coc::env::Env;
// use crate::coc::env::EnvEntry::{NSubst, NType};
// use crate::coc::ExprInner::{FnConstruct, FnDestruct, FnType, Let, Type, Var};
// use crate::coc::{Expr, SExpr, W};
// use by_address::ByAddress;
//
// pub fn brh<'a, M: Clone>((eo, so): SExpr<'a, M>) -> SExpr<'a, M> {
//     let mut args = Vec::new();
//
//     let mut e: &'a Expr<M> = eo;
//     let mut s: Env<M> = so.clone();
//
//     loop {
//         match &e.1 {
//             Type => {
//                 debug_assert!(args.is_empty());
//                 return (e, s);
//             }
//             Let(v, b) => {
//                 e = b;
//                 s = s.cons(NSubst((v, s.clone())));
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
//                     s = s.cons(NSubst(arg));
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
// pub fn br<M: Clone>(e: SExpr<M>) -> Expr<M> {
//     let (e, s) = brh(e);
//     Expr(e.0, match &e.1 {
//         Type => Type,
//         Let(_, _) => unreachable!(),
//         Var(i) => Var(*i),
//         FnType(a, b) => FnType(
//             W::new(br((a, s.clone()))),
//             W::new(br((b, s.cons(NType(a))))),
//         ),
//         FnConstruct(a, b) => FnConstruct(
//             W::new(br((a, s.clone()))),
//             W::new(br((b, s.cons(NType(a))))),
//         ),
//         FnDestruct(f, a) => FnDestruct(W::new(br((f, s.clone()))), W::new(br((a, s.clone())))),
//     })
// }
//
// pub fn beq<M: Clone>(e1: SExpr<M>, e2: SExpr<M>) -> Result<(), ()> {
//     match (brh(e1), brh(e2)) {
//         ((Type, _), (Type, _)) => Ok(()),
//         ((Var(i), s1), (Var(j), s2)) if ByAddress(&s1[*i]) == ByAddress(&s2[*j]) => Ok(()),
//         ((FnType(a1, b1), s1), (FnType(a2, b2), s2)) => {
//             beq((&*a1, s1.clone()), (&*a2, s2.clone()))?;
//             beq((&*b1, s1.cons(NType(a1))), (&*b2, s2.cons(NType(a2))))?;
//             Ok(())
//         }
//         ((FnConstruct(a1, b1), s1), (FnConstruct(a2, b2), s2)) => {
//             beq((&*a1, s1.clone()), (&*a2, s2.clone()))?;
//             beq((&*b1, s1.cons(NType(a1))), (&*b2, s2.cons(NType(a2))))?;
//             Ok(())
//         }
//         ((FnDestruct(a1, b1), s1), (FnDestruct(a2, b2), s2)) => {
//             beq((&*a1, s1.clone()), (&*a2, s2.clone()))?;
//             beq((&*b1, s1), (&*b2, s2))?;
//             Ok(())
//         }
//         (_, _) => Err(()),
//     }
// }
//
// pub fn shift_free<M: Clone>(e: Expr<M>, d: isize) -> Expr<M> {
//     fn sub<M: Clone>(e: &Expr<M>, d: isize, from: usize) -> Expr<M> {
//         match e {
//             Type => Type,
//             Let(v, b) => Let(W::new(sub(v, d, from)), W::new(sub(b, d, from + 1))),
//             Var(i) => {
//                 if *i >= from {
//                     Var(i.checked_add_signed(d).unwrap())
//                 } else {
//                     Var(*i)
//                 }
//             }
//             FnType(a, b) => FnType(W::new(sub(a, d, from)), W::new(sub(b, d, from + 1))),
//             FnConstruct(a, b) => FnConstruct(W::new(sub(a, d, from)), W::new(sub(b, d, from + 1))),
//             FnDestruct(f, a) => FnDestruct(W::new(sub(f, d, from)), W::new(sub(a, d, from))),
//         }
//     }
//     sub(&e, d, 0)
// }
