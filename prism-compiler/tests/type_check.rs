use bumpalo::Bump;
use prism_compiler::lang::env::Env;
use prism_compiler::lang::error::TypeResultExt;
use prism_compiler::lang::PrismEnv;
use prism_compiler::parser::parse_prism_in_env;
use prism_parser::core::cache::Allocs;
use prism_parser::error::aggregate_error::ParseResultExt;
use test_each_file::test_each_file;

fn test_ok([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input_str, rest) = rest.split_once("### Eval\n").unwrap();
    let (_eval, expected_typ) = rest.split_once("### Type\n").unwrap();

    let bump = Bump::new();
    let mut env = PrismEnv::new(Allocs::new(&bump));
    let input = parse_prism_in_env(input_str, &mut env).unwrap_or_eprint();
    let typ = env.type_check(input).unwrap_or_eprint(&mut env, input_str);

    let expected_typ = parse_prism_in_env(expected_typ, &mut env).unwrap_or_eprint();
    env.type_check(expected_typ)
        .unwrap_or_eprint(&mut env, input_str);

    assert!(
        env.is_beta_equal(typ, &Env::new(), expected_typ, &Env::new()),
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
    let input = parse_prism_in_env(test, &mut env).unwrap_or_eprint();

    if let Ok(typ) = env.type_check(input) {
        eprint!(        "Expected type checking to fail:\n\n------\n{}\n------ Term reduces to -->\n{}\n------\n\n------\n{}\n------ Type of term reduces to -->\n{}\n------\n\n.",
                        env.index_to_sm_string(input),
                        env.index_to_br_string(input),
                        env.index_to_sm_string(typ),
                        env.index_to_br_string(typ));
        panic!()
    }
}

// test_each_file! { for ["test"] in "prism-compiler/programs/type_check_fails" as type_check_fails => test_fail }

#[test]
fn placeholder() {}
