use std::rc::Rc;
use rpds::Vector;
use crate::coc::EnvEntry::{NSubst, NType};

pub enum Expr {
    Type,
    Let(WExpr, WExpr),
    Var(usize),
    FnType(WExpr, WExpr),
    FnConstruct(WExpr, WExpr),
    FnDestruct(WExpr, WExpr),
}

pub type WExpr = Rc<Expr>;

#[derive(Clone)]
pub enum EnvEntry {
    NType(WExpr),
    NSubst(SExpr),
}

pub type Env = Vector<EnvEntry>;

pub type SExpr = (WExpr, Env);

pub fn brh((eo, so): &SExpr) -> SExpr {
    let mut args = Vec::new();

    let mut e = eo;
    let mut s = so.clone();

    loop {
        match &**e {
            Expr::Type => {
                debug_assert!(args.is_empty());
                return (e.clone(), s);
            }
            Expr::Let(v, b) => {
                e = b;
                s.push_back_mut(NSubst((v.clone(), s.clone())));
            }
            Expr::Var(i) => {
                match &s[*i] {
                    NType(_) => {}
                    NSubst((ne, ns)) => {
                        e = ne;
                        // s = ns.clone();
                    }
                }
            }
            Expr::FnType(_, _) => {
                debug_assert!(args.is_empty());
                return (e.clone(), s);
            }
            Expr::FnConstruct(_, b) => match args.pop() {
                None => return (e.clone(), s.clone()),
                Some(arg) => {
                    e = b;
                    s.push_back_mut(NSubst(arg));
                }
            }
            Expr::FnDestruct(f, a) => {
                e = f;
                args.push((a.clone(), s.clone()));
            }
        }
    }
}

// pub fn brh(e_orig: &SExpr) -> SExpr {
//     let mut args = Vec::new();
//
//     let mut e = &e_orig.0;
//     let mut s = e_orig.1.clone();
//
//     loop {
//         match &**e {
//             Expr::Type => {
//                 debug_assert!(args.is_empty());
//                 return (e.clone(), s);
//             }
//             Expr::Let(v, b) => {
//                 e = b;
//                 s.push_back_mut(NSubst((v.clone(), s.clone())));
//             }
//             Expr::Var(i) => {
//                 match &s[*i] {
//                     NType(_) => {}
//                     NSubst((ne, ns)) => {
//                         e = ne;
//                         s = ns.clone();
//                     }
//                 }
//             }
//             Expr::FnType(_, _) => {
//                 debug_assert!(args.is_empty());
//                 return (e.clone(), s);
//             }
//             Expr::FnConstruct(_, b) => match args.pop() {
//                 None => return (e.clone(), s.clone()),
//                 Some(arg) => {
//                     e = b;
//                     s.push_back_mut(NSubst(arg));
//                 }
//             }
//             Expr::FnDestruct(f, a) => {
//                 e = f;
//                 args.push((a.clone(), s.clone()));
//             }
//         }
//     }
// }