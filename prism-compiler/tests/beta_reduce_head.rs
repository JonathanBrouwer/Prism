use prism_compiler::lang::PrismDb;
use prism_compiler::lang::env::DbEnv;
use test_each_file::test_each_file;

fn test([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input_str, rest) = rest.split_once("### Eval\n").unwrap();
    let (eval_str, _expected_typ) = rest.split_once("### Type\n").unwrap();

    let mut env = PrismDb::new();

    let input = env.load_test(input_str, "input");
    let (input, _) = env.parse_prism_file(input);
    let input = env.parsed_to_checked(input);
    env.assert_no_errors();

    let expected_eval = env.load_test(eval_str, "expected_eval");
    let (expected_eval, _) = env.parse_prism_file(expected_eval);
    let expected_eval = env.parsed_to_checked(expected_eval);
    env.assert_no_errors();

    assert!(
        env.is_beta_equal(input, &DbEnv::default(), expected_eval, &DbEnv::default()),
        "Expected terms to be equal under beta equality:\n\n------\n{}\n------ Reduces to -->\n{}\n------\n\n------\n{}\n------ Reduces to -->\n{}\n------\n\n.",
        env.index_to_sm_string(input),
        env.index_to_br_string(input, &DbEnv::default()),
        env.index_to_sm_string(expected_eval),
        env.index_to_br_string(expected_eval, &DbEnv::default()),
    );
}

test_each_file! { for ["test"] in "prism-compiler/programs/ok" => test }

#[test]
fn placeholder() {}
