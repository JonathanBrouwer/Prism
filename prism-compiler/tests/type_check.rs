use prism_compiler::lang::PrismDb;
use prism_compiler::lang::env::DbEnv;
use prism_compiler::type_check::TypecheckPrismEnv;
use test_each_file::test_each_file;

fn test_ok([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input_str, rest) = rest.split_once("### Eval\n").unwrap();
    let (_eval, expected_typ_str) = rest.split_once("### Type\n").unwrap();

    let mut env = PrismDb::new();

    let input = env.load_test(input_str, "input");
    let (input, _) = env.parse_prism_file(input);
    let typ = env.type_check(input);

    let expected_typ = env.load_test(expected_typ_str, "expected_typ");
    let (expected_typ, _) = env.parse_prism_file(expected_typ);

    env.assert_no_errors();

    let mut env = TypecheckPrismEnv::new(&mut env);
    assert!(
        env.is_beta_equal(typ, &DbEnv::default(), expected_typ, &DbEnv::default()),
        "Unexpected type of term:\n\n------\n{}\n------ Term reduces to -->\n{}\n------\n\n------\n{}\n------ Type of term reduces to -->\n{}\n------\n\n------\n{}\n------ Expected type reduces to -->\n{}\n------\n\n.",
        env.db.index_to_string(input),
        env.db.index_to_br_string(input, &DbEnv::default()),
        env.db.index_to_sm_string(typ),
        env.db.index_to_br_string(typ, &DbEnv::default()),
        env.db.index_to_sm_string(expected_typ),
        env.db.index_to_br_string(expected_typ, &DbEnv::default()),
    );
}

test_each_file! { for ["test"] in "prism-compiler/programs/ok" as ok => test_ok }

fn test_fail([test]: [&str; 1]) {
    let mut env = PrismDb::new();
    let input = env.load_test(test, "input");
    let (input, _) = env.parse_prism_file(input);
    let typ = env.type_check(input);

    if env.errors.is_empty() {
        eprint!(
            "Expected type checking to fail:\n\n------\n{}\n------ Term reduces to -->\n{}\n------\n\n------\n{}\n------ Type of term reduces to -->\n{}\n------\n\n.",
            env.index_to_sm_string(input),
            env.index_to_br_string(input, &DbEnv::default()),
            env.index_to_sm_string(typ),
            env.index_to_br_string(typ, &DbEnv::default())
        );
        panic!()
    }
}

test_each_file! { for ["test"] in "prism-compiler/programs/type_check_fails" as type_check_fails => test_fail }

#[test]
fn placeholder() {}
