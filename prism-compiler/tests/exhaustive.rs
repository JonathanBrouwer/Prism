use exhaustive::exhaustive_test;
use prism_compiler::lang::exhaustive::ExprWithEnv;

#[exhaustive_test(9)]
fn test_exhaustive(ExprWithEnv(mut env, root): ExprWithEnv) {
    let _ = env.type_check(root);
}
