use prism_compiler::coc::env::{Env, EnvEntry};
use prism_compiler::coc::{PartialExpr, TcEnv};
use prism_compiler::parse_prism_in_env;

#[test]
fn test_inference_in_scopes() {
    let mut env = TcEnv::new();
    
    let t1 = parse_prism_in_env("let Type; _", &mut env).unwrap();
    let t2 = parse_prism_in_env("#0", &mut env).unwrap();

    let tid = env.insert_union_index(PartialExpr::Type);
    let id = env.new_tc_id();
    let s = Env::new().cons(EnvEntry::CType(id, tid));
    let mut errors = Vec::new();
    env.expect_beq(t1, t2, &s, &mut errors);
    
    assert!(errors.is_empty());
    assert!(env.is_beta_equal(t1, &s, t2, &s));
}

#[test]
fn free_chains() {
    let mut env = TcEnv::new();
    
    let v1 = env.insert_union_index(PartialExpr::Type);
    let v2 = env.insert_union_index(PartialExpr::Free);
    let v3 = env.insert_union_index(PartialExpr::Free);

    let mut errors = Vec::new();
    env.expect_beq(v2, v3, &Env::new(), &mut errors);
    env.expect_beq(v1, v2, &Env::new(), &mut errors);

    assert!(errors.is_empty());
    assert!(env.is_beta_equal(v1, &Env::new(), v3, &Env::new()));
}