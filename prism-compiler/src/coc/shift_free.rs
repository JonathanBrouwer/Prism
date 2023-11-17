use prism_parser::parser::parser_instance::Arena;
use crate::coc::Expr;
use crate::coc::type_check::TcEnv;

impl<'arn> TcEnv<'arn> {
    // pub fn shift_free(&mut self, e: &'arn Expr<'arn>, d: isize) -> &'arn Expr<'arn> {
    //     self.shift_free_sub(&e, d, 0)
    // }
    //
    // fn shift_free_sub(&mut self, e: &'arn Expr<'arn>, d: isize, from: usize) -> &'arn Expr<'arn> {
    //     let v = match e {
    //         Expr::Type => return e,
    //         Expr::Let(v, b) => Expr::Let(self.shift_free_sub(v, d, from), self.shift_free_sub(b, d, from + 1)),
    //         Expr::Var(i) => {
    //             if *i >= from {
    //                 Expr::Var(i.checked_add_signed(d).unwrap())
    //             } else {
    //                 Expr::Var(*i)
    //             }
    //         }
    //         Expr::FnType(a, b) => Expr::FnType(self.shift_free_sub(a, d, from), self.shift_free_sub(b, d, from + 1)),
    //         Expr::FnConstruct(a, b) => Expr::FnConstruct(self.shift_free_sub(a, d, from), self.shift_free_sub(b, d, from + 1)),
    //         Expr::FnDestruct(f, a) => Expr::FnDestruct(self.shift_free_sub(f, d, from), self.shift_free_sub(a, d, from)),
    //     };
    //     self.arena.alloc(v)
    // }
}