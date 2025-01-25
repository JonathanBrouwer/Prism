use bumpalo::Bump;
use prism_compiler::lang::PrismEnv;
use prism_compiler::lang::env::Env;
use prism_compiler::parser::parse_prism_in_env;
use prism_parser::core::cache::Allocs;
use prism_parser::error::aggregate_error::ParseResultExt;
use test_each_file::test_each_file;

fn test([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input, rest) = rest.split_once("### Eval\n").unwrap();
    let (eval, expected_typ) = rest.split_once("### Type\n").unwrap();

    check(input);
    check(eval);
    check(expected_typ);
}

fn check(input: &str) {
    let bump = Bump::new();
    let mut env = PrismEnv::new(Allocs::new(&bump));
    let input = parse_prism_in_env(input, &mut env).unwrap_or_eprint();
    let _ = env.type_check(input);
    let sm = env.beta_reduce(input);

    assert!(
        env.is_beta_equal(input, &Env::new(), sm, &Env::new()),
        "Expected terms to be equal under beta equality:\n\n------\n{}\n------ Reduces to -->\n{}\n------\n\n------\n{}\n------ Reduces to -->\n{}\n------\n\n.",
        env.index_to_sm_string(input),
        env.index_to_br_string(input),
        env.index_to_sm_string(sm),
        env.index_to_br_string(sm),
    );
}

test_each_file! { for ["test"] in "prism-compiler/programs/ok" => test }

#[test]
fn placeholder() {}
