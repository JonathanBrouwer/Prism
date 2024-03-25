use prism_compiler::parse_prism_in_env;
use test_each_file::test_each_file;
use prism_compiler::coc::env::Env;
use prism_compiler::coc::TcEnv;

fn test([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input, rest) = rest.split_once("### Eval\n").unwrap();
    let (eval, _expected_typ) = rest.split_once("### Type\n").unwrap();

    let mut env = TcEnv::new();
    let input = parse_prism_in_env(input, &mut env).expect("Failed to parse input");
    let expected_eval = parse_prism_in_env(eval, &mut env).expect("Failed to parse input");

    assert!(
        env.beq(input, &Env::new(), expected_eval, &Env::new()),
        "Expected terms to be equal under beta equality:\n\n------\n{}\n------ Reduces to -->\n{}\n------\n\n------\n{}\n------ Reduces to -->\n{}\n------\n\n.",
        env.index_to_string(input, false),
        env.index_to_string(input, true),
        env.index_to_string(expected_eval, false),
        env.index_to_string(expected_eval, true),
    ); 
} 

test_each_file! { for ["test"] in "prism-compiler/programs/" => test }


#[test]
fn placeholder() {

}