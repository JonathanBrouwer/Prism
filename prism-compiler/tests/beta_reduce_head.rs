use prism_compiler::coc::env::Env;
use prism_compiler::coc::TcEnv;
use prism_compiler::parse_prism_in_env;
use test_each_file::test_each_file;

fn test([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input, rest) = rest.split_once("### Eval\n").unwrap();
    let (eval, _expected_typ) = rest.split_once("### Type\n").unwrap();

    let mut env = TcEnv::new();
    let input = parse_prism_in_env(input, &mut env).expect("Failed to parse input");
    let expected_eval = parse_prism_in_env(eval, &mut env).expect("Failed to parse input");

    assert!(
        env.is_beta_equal(input, &Env::new(), expected_eval, &Env::new()),
        "Expected terms to be equal under beta equality:\n\n------\n{}\n------ Reduces to -->\n{}\n------\n\n------\n{}\n------ Reduces to -->\n{}\n------\n\n.",
        env.index_to_sm_string(input),
        env.index_to_br_string(input),
        env.index_to_sm_string(expected_eval),
        env.index_to_br_string(expected_eval),
    );
}

test_each_file! { for ["test"] in "prism-compiler/programs/ok" => test }

#[test]
fn placeholder() {}
