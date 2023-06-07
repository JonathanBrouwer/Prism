use crate::coc::env::Env;
use crate::coc::env::EnvEntry::{NSubst, NType};
use crate::coc::{beta, Expr, W};
use crate::coc::Expr::{FnConstruct, FnDestruct, FnType, Let, Type, Var};

pub fn tc<'a>(e: &'a Expr, s: &Env<'a>) -> Result<Expr, ()> {
    match e {
        Type => Ok(Type),
        Let(v, b) => {
            tc(v, s)?;
            let bt = tc(b, &s.cons(NSubst((v, s.clone()))))?;
            Ok(beta::shift_free(bt, -1))
        }
        Var(i) => Ok(beta::shift_free(
            match &s[*i] {
                NType(e) => (**e).clone(),
                NSubst((e, s)) => tc(e, s)?,
            },
            (i + 1) as isize,
        )),
        FnType(a, b) => {
            let at = tc(a, s)?;
            beta::beq((&at, s.clone()), (&Type, Env::new()))?;
            let bt = tc(b, &s.cons(NType(a)))?;
            beta::beq((&bt, s.clone()), (&Type, Env::new()))?;
            Ok(Type)
        }
        FnConstruct(a, b) => {
            let at = tc(a, s)?;
            beta::beq((&at, s.clone()), (&Type, Env::new()))?;
            let a = beta::br((a, s.clone()));
            let bt = tc(b, &s.cons(NType(&a)))?;
            Ok(FnType(W::new(a), W::new(bt)))
        }
        FnDestruct(f, a) => {
            let ft = tc(f, s)?;
            let at = tc(a, s)?;
            let x = match beta::brh((&ft, Env::new())) {
                (FnType(da, db), sf) => {
                    beta::beq((&at, Env::new()), (da, sf.clone()))?;
                    Ok(beta::br((db, sf.cons(NSubst((a, s.clone()))))))
                }
                _ => Err(()),
            };
            x
        }
    }
}
