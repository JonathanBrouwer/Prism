use std::collections::VecDeque;
use prism_compiler::lang::{PartialExpr, TcEnv, UnionIndex};
use std::fmt::{Debug, Formatter};

// pub struct ExprWithEnv(pub TcEnv, pub UnionIndex);
//
// impl Exhaustive for ExprWithEnv {
//     fn generate(u: &mut DataSourceTaker) -> exhaustive::Result<Self> {
//         let mut env = TcEnv::default();
//         let idx = arbitrary_rec(0, &mut env, u)?;
//         Ok(ExprWithEnv(env, idx))
//     }
// }
//
// fn arbitrary_rec(
//     scope_size: usize,
//     env: &mut TcEnv,
//     u: &mut DataSourceTaker,
// ) -> exhaustive::Result<UnionIndex> {
//     let expr = match u.choice(6)? {
//         0 => PartialExpr::Type,
//         1 if scope_size > 0 => PartialExpr::DeBruijnIndex(u.choice(scope_size)?),
//         1 if scope_size == 0 => PartialExpr::Type,
//         2 => PartialExpr::Free,
//         3 => PartialExpr::FnType(
//             arbitrary_rec(scope_size, env, u)?,
//             arbitrary_rec(scope_size + 1, env, u)?,
//         ),
//         4 => PartialExpr::FnConstruct(
//             arbitrary_rec(scope_size, env, u)?,
//             arbitrary_rec(scope_size + 1, env, u)?,
//         ),
//         5 => PartialExpr::FnDestruct(
//             arbitrary_rec(scope_size, env, u)?,
//             arbitrary_rec(scope_size, env, u)?,
//         ),
//         _ => unreachable!(),
//     };
//
//     Ok(env.store_test(expr))
// }
//
// impl Debug for ExprWithEnv {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.0.index_to_string(self.1))
//     }
// }
//
// #[exhaustive_test(8)]
// fn test_exhaustive(ExprWithEnv(mut env, root): ExprWithEnv) {
//
//
//     let _ = env.type_check(root);
// }

const MAX_DEPTH: usize = 4;

#[test]
fn test_exhaustive() {
    let mut env = TcEnv::default();
    let mut stack = vec![env.store_test(PartialExpr::Free)];

    let mut i = 0;
    // loop {
    //
    // }
}

fn next(i: usize, env: &mut TcEnv, stack: &mut Vec<UnionIndex>) {
    match env.values[*stack[i]] {
        PartialExpr::Type => {}
        PartialExpr::Let(_, _) => {}
        PartialExpr::DeBruijnIndex(_) => {}
        PartialExpr::FnType(_, _) => {}
        PartialExpr::FnConstruct(_, _) => {}
        PartialExpr::FnDestruct(_, _) => {}
        PartialExpr::Free => {}
        PartialExpr::Shift(_, _) => {}
        PartialExpr::TypeAssert(_, _) => {}
    }
}