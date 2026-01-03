use prism_compiler::lang::PrismDb;
use prism_compiler::lang::env::DbEnv;
use prism_compiler::type_check::TypecheckPrismEnv;
use test_each_file::test_each_file;

fn test([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input, rest) = rest.split_once("### Eval\n").unwrap();
    let (eval, expected_typ) = rest.split_once("### Type\n").unwrap();

    check(input);
    check(eval);
    check(expected_typ);
}

fn check(input_str: &str) {
    let mut env = PrismDb::new();
    let input = env.load_test(input_str, "input");
    let (input, _) = env.parse_prism_file(input);

    let mut env = TypecheckPrismEnv::new(&mut env);
    let sm = env.beta_reduce(input, &DbEnv::default());
    env.db.assert_no_errors();

    assert!(
        env.is_beta_equal(input, &DbEnv::default(), sm, &DbEnv::default()),
        "Expected terms to be equal under beta equality:\n\n------\n{}\n------ Reduces to -->\n{}\n------\n\n------\n{}\n------ Reduces to -->\n{}\n------\n\n.",
        env.db.index_to_sm_string(input),
        env.db.index_to_br_string(input, &DbEnv::default()),
        env.db.index_to_sm_string(sm),
        env.db.index_to_br_string(sm, &DbEnv::default()),
    );
}

test_each_file! { for ["test"] in "prism-compiler/programs/ok" => test }

#[test]
fn placeholder() {}
