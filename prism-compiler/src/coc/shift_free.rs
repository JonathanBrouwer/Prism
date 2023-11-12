use prism_parser::parser::parser_instance::Arena;
use crate::coc::Expr;

pub fn shift_free<'arn>(e: &'arn Expr<'arn>, d: isize, arena: &'arn Arena<Expr<'arn>>) -> &'arn Expr<'arn> {
    fn sub<'arn>(e: &'arn Expr<'arn >, d: isize, from: usize, arena: &'arn Arena<Expr<'arn>>) -> &'arn Expr<'arn > {
        let v = match e {
            Expr::Type => return e,
            Expr::Let(v, b) => Expr::Let(sub(v, d, from, arena), sub(b, d, from + 1, arena)),
            Expr::Var(i) => {
                if *i >= from {
                    Expr::Var(i.checked_add_signed(d).unwrap())
                } else {
                    Expr::Var(*i)
                }
            }
            Expr::FnType(a, b) => Expr::FnType(sub(a, d, from, arena), sub(b, d, from + 1, arena)),
            Expr::FnConstruct(a, b) => Expr::FnConstruct(sub(a, d, from, arena), sub(b, d, from + 1, arena)),
            Expr::FnDestruct(f, a) => Expr::FnDestruct(sub(f, d, from, arena), sub(a, d, from, arena)),
        };
        arena.alloc(v)
    }
    sub(&e, d, 0, arena)
}