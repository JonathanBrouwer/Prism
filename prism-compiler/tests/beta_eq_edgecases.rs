use prism_compiler::lang::env::{Env, EnvEntry};
use prism_compiler::lang::{PartialExpr, TcEnv};
use prism_compiler::parser::parse_prism_in_env;
use prism_parser::error::aggregate_errors::ResultExt;

#[test]
fn test_inference_in_scopes() {
    let mut env = TcEnv::new();

    let t1 = parse_prism_in_env("let Type; _", &mut env).unwrap_or_eprint();
    let t2 = parse_prism_in_env("#0", &mut env).unwrap_or_eprint();

    let tid = env.store_test(PartialExpr::Type);
    let id = env.new_tc_id();
    let s = Env::new().cons(EnvEntry::CType(id, tid));
    env.expect_beq(t1, t2, &s);

    assert!(env.errors.is_empty());
    assert!(env.is_beta_equal(t1, &s, t2, &s));
}

#[test]
fn free_chains() {
    let mut env = TcEnv::new();

    let v1 = env.store_test(PartialExpr::Type);
    let v2 = env.store_test(PartialExpr::Free);
    let v3 = env.store_test(PartialExpr::Free);

    env.expect_beq(v2, v3, &Env::new());
    env.expect_beq(v1, v2, &Env::new());

    assert!(env.errors.is_empty());
    assert!(env.is_beta_equal(v1, &Env::new(), v3, &Env::new()));
}

#[test]
fn free_chains_long() {
    let mut env = TcEnv::new();

    let v1 = env.store_test(PartialExpr::Type);
    let v2 = env.store_test(PartialExpr::Free);
    let v3 = env.store_test(PartialExpr::Free);
    let v4 = env.store_test(PartialExpr::Free);
    let v5 = env.store_test(PartialExpr::Free);
    let v6 = env.store_test(PartialExpr::Free);
    let v7 = env.store_test(PartialExpr::Free);

    env.expect_beq(v6, v7, &Env::new());
    env.expect_beq(v3, v4, &Env::new());
    env.expect_beq(v5, v6, &Env::new());
    env.expect_beq(v2, v3, &Env::new());
    env.expect_beq(v1, v2, &Env::new());
    env.expect_beq(v4, v5, &Env::new());

    assert!(env.errors.is_empty());
    assert!(env.is_beta_equal(v1, &Env::new(), v7, &Env::new()));
}

#[test]
fn dumb_shift() {
    let mut env = TcEnv::new();

    // Construct env
    let tid = env.store_test(PartialExpr::Type);
    let id = env.new_tc_id();
    let s = Env::new().cons(EnvEntry::CType(id, tid));
    
    // Construct left side
    let left = env.store_test(PartialExpr::Var(0));
    
    // Construct right side
    let right_inner = env.store_test(PartialExpr::Free);
    let right = env.store_test(PartialExpr::Shift(right_inner, 1));
    
    // Test
    env.expect_beq(left, right, &s);
    assert!(env.errors.len() > 0);
}