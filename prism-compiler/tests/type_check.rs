use prism_compiler::coc::env::Env;
use prism_compiler::coc::TcEnv;
use prism_compiler::parse_prism_in_env;
use test_each_file::test_each_file;
use exhaustive::exhaustive_test;
use prism_compiler::coc::arbitrary::ExprWithEnv;

fn test_ok([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input, rest) = rest.split_once("### Eval\n").unwrap();
    let (_eval, expected_typ) = rest.split_once("### Type\n").unwrap();

    let mut env = TcEnv::new();
    let input = parse_prism_in_env(input, &mut env).expect("Failed to parse input");
    let typ = env.type_check(input).unwrap();

    let expected_typ = parse_prism_in_env(expected_typ, &mut env).expect("Failed to parse input");

    assert!(
        env.is_beta_equal(typ, &Env::new(), expected_typ, &Env::new()),
        "Unexpected type of term:\n\n------\n{}\n------ Term reduces to -->\n{}\n------\n\n------\n{}\n------ Type of term reduces to -->\n{}\n------\n\n------\n{}\n------ Expected type reduces to -->\n{}\n------\n\n.",
        env.index_to_sm_string(input),
        env.index_to_br_string(input),
        env.index_to_sm_string(typ),
        env.index_to_br_string(typ),
        env.index_to_sm_string(expected_typ),
        env.index_to_br_string(expected_typ),
    ); 
}

test_each_file! { for ["test"] in "prism-compiler/programs/ok" as ok => test_ok }

fn test_fail([test]: [&str; 1]) {
    let mut env = TcEnv::new();
    let input = parse_prism_in_env(test, &mut env).expect("Failed to parse input");
    let typ = env.type_check(input);

    assert!(
        typ.is_err(),
        "Expected type checking to fail:\n\n------\n{}\n------ Term reduces to -->\n{}\n------\n\n------\n{}\n------ Type of term reduces to -->\n{}\n------\n\n.",
        env.index_to_sm_string(input),
        env.index_to_br_string(input),
        env.index_to_sm_string(*typ.as_ref().unwrap()),
        env.index_to_br_string(*typ.as_ref().unwrap()),
    );
}   

test_each_file! { for ["test"] in "prism-compiler/programs/type_check_fails" as fails => test_fail }

#[exhaustive_test(3)]
fn test_exhaustive(ExprWithEnv(mut env, root): ExprWithEnv) {
    match env.type_check(root) {
        Ok(ty) => {
            let ty_ty = env.type_check(ty).unwrap();
            assert!(env.is_beta_equal(ty_ty, &Env::new(), TcEnv::type_type(), &Env::new()));
        }
        Err(e) => {
            assert!(e.len() > 0);
        }
    }
}

#[test]
fn placeholder() {}
