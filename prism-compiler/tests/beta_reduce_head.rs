use prism_compiler::lang::env::Env;
use prism_compiler::lang::TcEnv;
use test_each_file::test_each_file;
use prism_compiler::parser::parse_prism_in_env;
use prism_parser::error::aggregate_error::ParseResultExt;

fn test([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input, rest) = rest.split_once("### Eval\n").unwrap();
    let (eval, _expected_typ) = rest.split_once("### Type\n").unwrap();

    let mut env = TcEnv::new();
    let input = parse_prism_in_env(input, &mut env).unwrap_or_eprint();
    let expected_eval = parse_prism_in_env(eval, &mut env).unwrap_or_eprint();

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
