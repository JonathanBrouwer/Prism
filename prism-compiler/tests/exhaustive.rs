use prism_compiler::lang::{PartialExpr, TcEnv, UnionIndex};

#[test]
fn test_exhaustive() {
    iter_exhaustive(6, false, |env, root| {
        env.type_check(root).is_ok()
    })
}

fn iter_exhaustive(max_depth: usize, continue_when_fail: bool, mut f: impl FnMut(&mut TcEnv, UnionIndex) -> bool) {
    let mut env = TcEnv::default();
    let root = env.store_test(PartialExpr::Free);
    let mut env_size = vec![0];

    // Invariant: env.values[i+1..] is free
    let mut i = 0;
    loop {
        let len = env.values.len();

        // Check if type checks
        let is_ok = f(&mut env, root);

        // Reset free variables
        env.values.truncate(len);
        env.value_origins.truncate(len);
        env.values[i+1..].fill(PartialExpr::Free);
        env.reset();

        // Keep this partial expr if the result is ok
        if (is_ok || continue_when_fail) && i < max_depth && i + 1 < env.values.len() {
            i += 1;
        }

        // Go to the next value
        if !next(&mut i, &mut env, &mut env_size) {
            break
        }
    }
}

fn next(i: &mut usize, env: &mut TcEnv, env_size: &mut Vec<usize>) -> bool {
    loop {
        env.values[*i] = match env.values[*i] {
            PartialExpr::Free => PartialExpr::Type,
            PartialExpr::Type => PartialExpr::DeBruijnIndex(0),
            PartialExpr::DeBruijnIndex(idx) => {
                if idx + 1 < env_size[*i] {
                    PartialExpr::DeBruijnIndex(idx+1)
                } else {
                    env_size.push(env_size[*i]);
                    env_size.push(env_size[*i] + 1);
                    PartialExpr::Let(env.store_test(PartialExpr::Free), env.store_test(PartialExpr::Free))
                }
            }
            PartialExpr::Let(e1, e2) => PartialExpr::FnType(e1, e2),
            PartialExpr::FnType(e1, e2) => PartialExpr::FnConstruct(e1, e2),
            PartialExpr::FnConstruct(e1, e2) => {
                env_size[*e2] -= 1;
                PartialExpr::FnDestruct(e1, e2)
            },
            PartialExpr::FnDestruct(e1, e2) => PartialExpr::TypeAssert(e1, e2),
            PartialExpr::TypeAssert(_, _) => {
                env.values[*i] = PartialExpr::Free;
                env.values.pop().unwrap();
                env.values.pop().unwrap();
                env_size.pop().unwrap();
                env_size.pop().unwrap();
                if *i == 0 {
                    return false;
                }
                *i -= 1;
                continue
            }
            PartialExpr::Shift(_, _) => unreachable!(),
        };
        return true
    }
}

