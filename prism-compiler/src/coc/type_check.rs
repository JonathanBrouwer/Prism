use prism_parser::parser::parser_instance::Arena;
use crate::coc::env::Env;
use crate::coc::env::EnvEntry::{NSubst, NType};
use crate::coc::{beta, Expr};
use crate::coc::shift_free::shift_free;

pub fn tc<'arn>(e: &'arn Expr<'arn>, s: &Env<'arn>, arena: &'arn Arena<Expr<'arn>>) -> Result<&'arn Expr<'arn>, ()> {
    let t = match e {
        Expr::Type => Expr::Type,
        Expr::Let(v, b) => {
            let vt = tc(v, s, arena)?;
            let bt = tc(b, &s.cons(NSubst(vt, (v, s.clone()))), arena)?;
            return Ok(shift_free(bt, -1, arena))
        }
        Expr::Var(i) => return Ok(shift_free(
        s[*i].typ(),
            (i + 1) as isize,
            arena
        )),
        // Expr::FnType(a, b) => {
        //     let at = tc(a, s)?;
        //     beta::beq((&at, s.clone()), (&Type, Env::new()))?;
        //     let bt = tc(b, &s.cons(NType(a)))?;
        //     beta::beq((&bt, s.clone()), (&Type, Env::new()))?;
        //     Ok(Expr::Type)
        // }
        // Expr::FnConstruct(a, b) => {
        //     let at = tc(a, s)?;
        //     beta::beq((&at, s.clone()), (&Type, Env::new()))?;
        //     let a = beta::br((a, s.clone()));
        //     let bt = tc(b, &s.cons(NType(&a)))?;
        //     Ok(FnType(W::new(a), W::new(bt)))
        // }
        // Expr::FnDestruct(f, a) => {
        //     let ft = tc(f, s)?;
        //     let at = tc(a, s)?;
        //     let x = match beta::brh((&ft, Env::new())) {
        //         (FnType(da, db), sf) => {
        //             beta::beq((&at, Env::new()), (da, sf.clone()))?;
        //             Ok(beta::br((db, sf.cons(NSubst((a, s.clone()))))))
        //         }
        //         _ => Err(()),
        //     };
        //     x
        // }
        _ => unreachable!(),
    };
    Ok(arena.alloc(t))
}
