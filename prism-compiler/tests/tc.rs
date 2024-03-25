use prism_compiler::parse_prism_in_env;
use test_each_file::test_each_file;
use prism_compiler::coc::env::Env;
use prism_compiler::coc::TcEnv;

fn test([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input, rest) = rest.split_once("### Eval\n").unwrap();
    let (_eval, expected_typ) = rest.split_once("### Type\n").unwrap();

    let mut env = TcEnv::new();
    let input = parse_prism_in_env(input, &mut env).expect("Failed to parse input");
    let typ = env.type_check(input).unwrap();

    let expected_typ = parse_prism_in_env(expected_typ, &mut env).expect("Failed to parse input");

    assert!(
        false, //env.beq(typ, &Env::new(), expected_typ, &Env::new()),
        "Unexpected type of term:\n\n------\n{}\n------ Term reduces to -->\n{}\n------\n\n------\n{}\n------ Type of term reduces to -->\n{}\n------\n\n------\n{}\n------ Expected type reduces to -->\n{}\n------\n\n.",
        env.index_to_string(input, false),
        env.index_to_string(input, true),
        env.index_to_string(typ, false),
        env.index_to_string(typ, true),
        env.index_to_string(expected_typ, false),
        env.index_to_string(expected_typ, true),
    ); 
}

test_each_file! { for ["test"] in "prism-compiler/programs/" => test }


#[test]
fn placeholder() {

}