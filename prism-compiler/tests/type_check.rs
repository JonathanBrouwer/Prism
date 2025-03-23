use bumpalo::Bump;
use prism_compiler::lang::PrismEnv;
use prism_compiler::lang::env::DbEnv;
use prism_compiler::lang::error::TypeResultExt;
use prism_parser::core::allocs::Allocs;
use prism_parser::error::aggregate_error::ParseResultExt;
use test_each_file::test_each_file;

fn test_ok([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input_str, rest) = rest.split_once("### Eval\n").unwrap();
    let (_eval, expected_typ_str) = rest.split_once("### Type\n").unwrap();

    let bump = Bump::new();
    let mut env = PrismEnv::new(Allocs::new(&bump));

    let input = env.load_test(input_str, "input");
    let input = env.parse_file(input).unwrap_or_eprint();
    let input = env.parsed_to_checked(input);
    let typ = env.type_check(input).unwrap_or_eprint(&mut env);

    let expected_typ = env.load_test(expected_typ_str, "expected_typ");
    let expected_typ = env.parse_file(expected_typ).unwrap_or_eprint();
    let expected_typ = env.parsed_to_checked(expected_typ);

    assert!(
        env.is_beta_equal(typ, DbEnv::default(), expected_typ, DbEnv::default()),
        "Unexpected type of term:\n\n------\n{}\n------ Term reduces to -->\n{}\n------\n\n------\n{}\n------ Type of term reduces to -->\n{}\n------\n\n------\n{}\n------ Expected type reduces to -->\n{}\n------\n\n.",
        env.index_to_string(input),
        env.index_to_br_string(input),
        env.index_to_sm_string(typ),
        env.index_to_br_string(typ),
        env.index_to_sm_string(expected_typ),
        env.index_to_br_string(expected_typ),
    );
}

test_each_file! { for ["test"] in "prism-compiler/programs/ok" as ok => test_ok }

fn test_fail([test]: [&str; 1]) {
    let bump = Bump::new();
    let mut env = PrismEnv::new(Allocs::new(&bump));
    let input = env.load_test(test, "input");
    let input = env.parse_file(input).unwrap_or_eprint();
    let input = env.parsed_to_checked(input);

    if let Ok(typ) = env.type_check(input) {
        eprint!(
            "Expected type checking to fail:\n\n------\n{}\n------ Term reduces to -->\n{}\n------\n\n------\n{}\n------ Type of term reduces to -->\n{}\n------\n\n.",
            env.index_to_sm_string(input),
            env.index_to_br_string(input),
            env.index_to_sm_string(typ),
            env.index_to_br_string(typ)
        );
        panic!()
    }
}

test_each_file! { for ["test"] in "prism-compiler/programs/type_check_fails" as type_check_fails => test_fail }

#[test]
fn placeholder() {}
