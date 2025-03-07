use bumpalo::Bump;
use prism_compiler::lang::PrismEnv;
use prism_compiler::lang::env::DbEnv;
use prism_compiler::parser::parse_prism_in_env;
use prism_parser::core::allocs::Allocs;
use prism_parser::error::aggregate_error::ParseResultExt;
use test_each_file::test_each_file;

fn test([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input_str, rest) = rest.split_once("### Eval\n").unwrap();
    let (eval_str, _expected_typ) = rest.split_once("### Type\n").unwrap();

    let bump = Bump::new();
    let mut env = PrismEnv::new(Allocs::new(&bump));

    let input = parse_prism_in_env(input_str, &mut env).unwrap_or_eprint();
    let input = env.parsed_to_checked(input);

    let expected_eval = parse_prism_in_env(eval_str, &mut env).unwrap_or_eprint();
    let expected_eval = env.parsed_to_checked(expected_eval);

    assert!(
        env.is_beta_equal(input, DbEnv::default(), expected_eval, DbEnv::default()),
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
