use std::ops::Index;
use crate::coc::EnvEntry::{NSubst, NType};
use crate::coc::Expr::*;
use by_address::ByAddress;
use rpds::Vector;
use std::rc::Rc;

pub type W<T> = Rc<T>;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Expr {
    Type,
    Let(W<Self>, W<Self>),
    Var(usize),
    FnType(W<Self>, W<Self>),
    FnConstruct(W<Self>, W<Self>),
    FnDestruct(W<Self>, W<Self>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum EnvEntry<'a> {
    NType(&'a Expr),
    NSubst(SExpr<'a>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Env<'a>(Vector<EnvEntry<'a>>);

impl<'a> Env<'a> {
    pub fn new() -> Self {
        Self(Vector::new())
    }

    #[must_use]
    pub fn cons(&self, e: EnvEntry<'a>) -> Self {
        Env(self.0.push_back(e))
    }
}

impl<'a> Index<usize> for Env<'a> {
    type Output = EnvEntry<'a>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[self.0.len() - 1 - index]
    }
}

pub type SExpr<'a> = (&'a Expr, Env<'a>);

pub fn brh<'a>((eo, so): SExpr<'a>) -> SExpr<'a> {
    let mut args = Vec::new();

    let mut e: &'a Expr = eo;
    let mut s: Env = so.clone();

    loop {
        match e {
            Type => {
                debug_assert!(args.is_empty());
                return (e, s);
            }
            Let(v, b) => {
                e = b;
                s = s.cons(NSubst((v, s.clone())));
            }
            Var(i) => match &s[*i] {
                NType(_) => {
                    return if args.len() == 0 {
                        (e, s)
                    } else {
                        (eo, so.clone())
                    }
                }
                NSubst((ne, ns)) => {
                    e = ne;
                    s = ns.clone();
                }
            },
            FnType(_, _) => {
                debug_assert!(args.is_empty());
                return (e, s);
            }
            FnConstruct(_, b) => match args.pop() {
                None => return (e, s.clone()),
                Some(arg) => {
                    e = b;
                    s = s.cons(NSubst(arg));
                }
            },
            FnDestruct(f, a) => {
                e = f;
                args.push((a, s.clone()));
            }
        }
    }
}

pub fn br(e: SExpr) -> Expr {
    let (e, s) = brh(e);
    match e {
        Type => Type,
        Let(_, _) => unreachable!(),
        Var(i) => Var(*i),
        FnType(a, b) => FnType(
            W::new(br((a, s.clone()))),
            W::new(br((b, s.cons(NType(a))))),
        ),
        FnConstruct(a, b) => FnConstruct(
            W::new(br((a, s.clone()))),
            W::new(br((b, s.cons(NType(a))))),
        ),
        FnDestruct(f, a) => FnDestruct(W::new(br((f, s.clone()))), W::new(br((a, s.clone())))),
    }
}

pub fn beq(e1: SExpr, e2: SExpr) -> Result<(), ()> {
    match (brh(e1), brh(e2)) {
        ((Type, _), (Type, _)) => Ok(()),
        ((Var(i), s1), (Var(j), s2)) if ByAddress(&s1[*i]) == ByAddress(&s2[*j]) => Ok(()),
        ((FnType(a1, b1), s1), (FnType(a2, b2), s2)) => {
            beq((&*a1, s1.clone()), (&*a2, s2.clone()))?;
            beq(
                (&*b1, s1.cons(NType(a1))),
                (&*b2, s2.cons(NType(a2))),
            )?;
            Ok(())
        }
        ((FnConstruct(a1, b1), s1), (FnConstruct(a2, b2), s2)) => {
            beq((&*a1, s1.clone()), (&*a2, s2.clone()))?;
            beq(
                (&*b1, s1.cons(NType(a1))),
                (&*b2, s2.cons(NType(a2))),
            )?;
            Ok(())
        }
        ((FnDestruct(a1, b1), s1), (FnDestruct(a2, b2), s2)) => {
            beq((&*a1, s1.clone()), (&*a2, s2.clone()))?;
            beq((&*b1, s1), (&*b2, s2))?;
            Ok(())
        }
        (_, _) => Err(()),
    }
}

pub fn tc<'a>(e: &'a Expr, s: &Env<'a>) -> Result<Expr, ()> {
    match e {
        Type => Ok(Type),
        Let(v, b) => {
            tc(v, s)?;
            let bt = tc(b, &s.cons(NSubst((v, s.clone()))))?;
            Ok(shift_free(bt, -1))
        }
        Var(i) => Ok(shift_free(
            match &s[*i] {
                NType(e) => (**e).clone(),
                NSubst((e, s)) => tc(e, s)?,
            },
            (i + 1) as isize,
        )),
        FnType(a, b) => {
            let at = tc(a, s)?;
            beq((&at, s.clone()), (&Type, Env::new()))?;
            let bt = tc(b, &s.cons(NType(a)))?;
            beq((&bt, s.clone()), (&Type, Env::new()))?;
            Ok(Type)
        }
        FnConstruct(a, b) => {
            let at = tc(a, s)?;
            beq((&at, s.clone()), (&Type, Env::new()))?;
            let a = br((a, s.clone()));
            let bt = tc(b, &s.cons(NType(&a)))?;
            Ok(FnType(W::new(a), W::new(bt)))
        }
        FnDestruct(f, a) => {
            let ft = tc(f, s)?;
            let at = tc(a, s)?;
            let x = match brh((&ft, Env::new())) {
                (FnType(da, db), sf) => {
                    beq((&at, Env::new()), (da, sf.clone()))?;
                    Ok(br((db, sf.cons(NSubst((a, s.clone()))))))
                }
                _ => Err(()),
            };
            x
        }
    }
}

pub fn shift_free(e: Expr, d: isize) -> Expr {
    fn sub(e: &Expr, d: isize, from: usize) -> Expr {
        match e {
            Type => Type,
            Let(v, b) => Let(W::new(sub(v, d, from)), W::new(sub(b, d, from + 1))),
            Var(i) => {
                if *i >= from {
                    Var(i.checked_add_signed(d).unwrap())
                } else {
                    Var(*i)
                }
            }
            FnType(a, b) => FnType(W::new(sub(a, d, from)), W::new(sub(b, d, from + 1))),
            FnConstruct(a, b) => FnConstruct(W::new(sub(a, d, from)), W::new(sub(b, d, from + 1))),
            FnDestruct(f, a) => FnDestruct(W::new(sub(f, d, from)), W::new(sub(a, d, from))),
        }
    }
    sub(&e, d, 0)
}
