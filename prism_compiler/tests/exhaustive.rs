// use bumpalo::Bump;
// use prism_compiler::lang::{CoreIndex, CorePrismExpr, PrismEnv};
// use prism_parser::core::allocs::Allocs;
//
// #[test]
// #[ignore]
// fn test_exhaustive() {
//     iter_exhaustive(4, false, |env, root| env.type_check(root).is_ok())
// }
//
// fn iter_exhaustive(
//     max_depth: usize,
//     continue_when_fail: bool,
//     mut f: impl FnMut(&mut PrismEnv, CoreIndex) -> bool,
// ) {
//     let bump = OwnedAllocs::default();
//     let mut env = PrismEnv::new(Allocs::new(&bump));
//     let root = env.store_test(CorePrismExpr::Free);
//     let mut env_size = vec![0];
//
//     // Invariant: env.values[i+1..] is free
//     let mut i = 0;
//     for _count in 0.. {
//         let len = env.checked_values.len();
//
//         // Check if type checks
//         let is_ok = f(&mut env, root);
//
//         // Reset free variables
//         env.checked_values.truncate(len);
//         env.checked_origins.truncate(len);
//         env.checked_values[i + 1..].fill(CorePrismExpr::Free);
//         env.reset();
//
//         // Keep this partial expr if the result is ok
//         if (is_ok || continue_when_fail) && i < max_depth && i + 1 < env.checked_values.len() {
//             i += 1;
//         }
//
//         // Go to the next value
//         if !next(&mut i, &mut env, &mut env_size) {
//             break;
//         }
//     }
// }
//
// fn next(i: &mut usize, env: &mut PrismEnv, env_size: &mut Vec<usize>) -> bool {
//     loop {
//         env.checked_values[*i] = match env.checked_values[*i] {
//             CorePrismExpr::Free => CorePrismExpr::Type,
//             CorePrismExpr::Type => CorePrismExpr::DeBruijnIndex(0),
//             CorePrismExpr::DeBruijnIndex(idx) => {
//                 if idx + 1 < env_size[*i] {
//                     CorePrismExpr::DeBruijnIndex(idx + 1)
//                 } else {
//                     env_size.push(env_size[*i]);
//                     env_size.push(env_size[*i] + 1);
//                     CorePrismExpr::Let(
//                         env.store_test(CorePrismExpr::Free),
//                         env.store_test(CorePrismExpr::Free),
//                     )
//                 }
//             }
//             CorePrismExpr::Let(e1, e2) => CorePrismExpr::FnType(e1, e2),
//             CorePrismExpr::FnType(e1, _) => {
//                 env.checked_values.pop().unwrap();
//                 env_size.pop().unwrap();
//                 env_size[*e1] += 1;
//                 CorePrismExpr::FnConstruct(e1)
//             }
//             CorePrismExpr::FnConstruct(e1) => {
//                 env_size[*e1] -= 1;
//                 env_size.push(env_size[*i]);
//                 CorePrismExpr::FnDestruct(e1, env.store_test(CorePrismExpr::Free))
//             }
//             CorePrismExpr::FnDestruct(e1, e2) => CorePrismExpr::TypeAssert(e1, e2),
//             CorePrismExpr::TypeAssert(_, _) => {
//                 env.checked_values[*i] = CorePrismExpr::Free;
//                 env.checked_values.pop().unwrap();
//                 env.checked_values.pop().unwrap();
//                 env_size.pop().unwrap();
//                 env_size.pop().unwrap();
//                 if *i == 0 {
//                     return false;
//                 }
//                 *i -= 1;
//                 continue;
//             }
//             CorePrismExpr::Shift(_, _) => unreachable!(),
//             CorePrismExpr::GrammarValue(_) => unreachable!(),
//             CorePrismExpr::GrammarType => unreachable!(),
//         };
//         return true;
//     }
// }
