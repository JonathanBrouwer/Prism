use bumpalo::Bump;
use prism_compiler::lang::{PrismEnv, PrismExpr, UnionIndex};
use prism_parser::core::cache::Allocs;

#[test]
fn test_exhaustive() {
    iter_exhaustive(5, false, |env, root| env.type_check(root).is_ok())
}

fn iter_exhaustive(
    max_depth: usize,
    continue_when_fail: bool,
    mut f: impl FnMut(&mut PrismEnv, UnionIndex) -> bool,
) {
    let bump = Bump::new();
    let mut env = PrismEnv::new(Allocs::new(&bump));
    let root = env.store_test(PrismExpr::Free);
    let mut env_size = vec![0];

    // Invariant: env.values[i+1..] is free
    let mut i = 0;
    for _count in 0.. {
        let len = env.values.len();

        // println!("{count}");

        // if count == 1073864 {
        //     println!("{}", env.index_to_string(root));
        // }

        // Check if type checks
        let is_ok = f(&mut env, root);

        // Reset free variables
        env.values.truncate(len);
        env.value_origins.truncate(len);
        env.values[i + 1..].fill(PrismExpr::Free);
        env.reset();

        // Keep this partial expr if the result is ok
        if (is_ok || continue_when_fail) && i < max_depth && i + 1 < env.values.len() {
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
        env.values[*i] = match env.values[*i] {
            PrismExpr::Free => PrismExpr::Type,
            PrismExpr::Type => PrismExpr::DeBruijnIndex(0),
            PrismExpr::DeBruijnIndex(idx) => {
                if idx + 1 < env_size[*i] {
                    PrismExpr::DeBruijnIndex(idx + 1)
                } else {
                    env_size.push(env_size[*i]);
                    env_size.push(env_size[*i] + 1);
                    PrismExpr::Let(
                        "",
                        env.store_test(PrismExpr::Free),
                        env.store_test(PrismExpr::Free),
                    )
                }
            }
            PrismExpr::Let(_, e1, e2) => PrismExpr::FnType("", e1, e2),
            PrismExpr::FnType(_, e1, _) => {
                env.values.pop().unwrap();
                env_size.pop().unwrap();
                env_size[*e1] += 1;
                PrismExpr::FnConstruct("", e1)
            }
            PrismExpr::FnConstruct(_, e1) => {
                env_size[*e1] -= 1;
                env_size.push(env_size[*i]);
                PrismExpr::FnDestruct(e1, env.store_test(PrismExpr::Free))
            }
            PrismExpr::FnDestruct(e1, e2) => PrismExpr::TypeAssert(e1, e2),
            PrismExpr::TypeAssert(_, _) => {
                env.values[*i] = PrismExpr::Free;
                env.values.pop().unwrap();
                env.values.pop().unwrap();
                env_size.pop().unwrap();
                env_size.pop().unwrap();
                if *i == 0 {
                    return false;
                }
                *i -= 1;
                continue;
            }
            PrismExpr::Shift(_, _) => unreachable!(),
            PrismExpr::Name(_) => unreachable!(),
            PrismExpr::ShiftPoint(_, _) => unreachable!(),
            PrismExpr::ShiftTo(_, _, _) => unreachable!(),
            PrismExpr::ParserValue(_) => unreachable!(),
            PrismExpr::ParserValueType => unreachable!(),
            PrismExpr::ShiftToTrigger(_, _, _) => unreachable!(),
        };
        return true;
    }
}
