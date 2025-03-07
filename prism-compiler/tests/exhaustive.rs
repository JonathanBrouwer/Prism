use bumpalo::Bump;
use prism_compiler::lang::{CheckedIndex, CheckedPrismExpr, PrismEnv};
use prism_parser::core::allocs::Allocs;

#[test]
fn test_exhaustive() {
    iter_exhaustive(5, false, |env, root| env.type_check(root).is_ok())
}

fn iter_exhaustive(
    max_depth: usize,
    continue_when_fail: bool,
    mut f: impl FnMut(&mut PrismEnv, CheckedIndex) -> bool,
) {
    let bump = Bump::new();
    let mut env = PrismEnv::new(Allocs::new(&bump));
    let root = env.store_test(CheckedPrismExpr::Free);
    let mut env_size = vec![0];

    // Invariant: env.values[i+1..] is free
    let mut i = 0;
    for _count in 0.. {
        let len = env.checked_values.len();

        // Check if type checks
        let is_ok = f(&mut env, root);

        // Reset free variables
        env.checked_values.truncate(len);
        env.checked_origins.truncate(len);
        env.checked_values[i + 1..].fill(CheckedPrismExpr::Free);
        env.reset();

        // Keep this partial expr if the result is ok
        if (is_ok || continue_when_fail) && i < max_depth && i + 1 < env.checked_values.len() {
            i += 1;
        }

        // Go to the next value
        if !next(&mut i, &mut env, &mut env_size) {
            break;
        }
    }
}

fn next(i: &mut usize, env: &mut PrismEnv, env_size: &mut Vec<usize>) -> bool {
    loop {
        env.checked_values[*i] = match env.checked_values[*i] {
            CheckedPrismExpr::Free => CheckedPrismExpr::Type,
            CheckedPrismExpr::Type => CheckedPrismExpr::DeBruijnIndex(0),
            CheckedPrismExpr::DeBruijnIndex(idx) => {
                if idx + 1 < env_size[*i] {
                    CheckedPrismExpr::DeBruijnIndex(idx + 1)
                } else {
                    env_size.push(env_size[*i]);
                    env_size.push(env_size[*i] + 1);
                    CheckedPrismExpr::Let(
                        env.store_test(CheckedPrismExpr::Free),
                        env.store_test(CheckedPrismExpr::Free),
                    )
                }
            }
            CheckedPrismExpr::Let(e1, e2) => CheckedPrismExpr::FnType(e1, e2),
            CheckedPrismExpr::FnType(e1, _) => {
                env.checked_values.pop().unwrap();
                env_size.pop().unwrap();
                env_size[*e1] += 1;
                CheckedPrismExpr::FnConstruct(e1)
            }
            CheckedPrismExpr::FnConstruct(e1) => {
                env_size[*e1] -= 1;
                env_size.push(env_size[*i]);
                CheckedPrismExpr::FnDestruct(e1, env.store_test(CheckedPrismExpr::Free))
            }
            CheckedPrismExpr::FnDestruct(e1, e2) => CheckedPrismExpr::TypeAssert(e1, e2),
            CheckedPrismExpr::TypeAssert(_, _) => {
                env.checked_values[*i] = CheckedPrismExpr::Free;
                env.checked_values.pop().unwrap();
                env.checked_values.pop().unwrap();
                env_size.pop().unwrap();
                env_size.pop().unwrap();
                if *i == 0 {
                    return false;
                }
                *i -= 1;
                continue;
            }
            CheckedPrismExpr::Shift(_, _) => unreachable!(),
            CheckedPrismExpr::GrammarValue(_, _) => unreachable!(),
            CheckedPrismExpr::GrammarType => unreachable!(),
        };
        return true;
    }
}
