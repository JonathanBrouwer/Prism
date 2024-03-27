use prism_compiler::coc::env::{Env, EnvEntry};
use prism_compiler::coc::TcEnv;
use prism_compiler::parse_prism_in_env;

#[test]
fn test_inference_in_scopes() {
    let mut env = TcEnv::new();
    
    let t1 = parse_prism_in_env("let Type; _", &mut env).unwrap();
    let t2 = parse_prism_in_env("#0", &mut env).unwrap();
    
    let id = env.new_tc_id();
    let s = Env::new().cons(EnvEntry::RType(id));
    let mut errors = Vec::new();
    env.expect_beq(t1, t2, &s, &mut errors);
    
    assert!(errors.is_empty());
    assert!(env.is_beta_equal(t1, &s, t2, &s));
}