use prism_compiler::parse_prism_in_env;
use test_each_file::test_each_file;
use prism_compiler::coc::env::Env;
use prism_compiler::coc::TcEnv;

fn test([input, output]: [&str; 2]) {
    let mut env = TcEnv::new();
    let input = parse_prism_in_env(input, &mut env).expect("Failed to parse input");
    let output = parse_prism_in_env(output, &mut env).expect("Failed to parse input");

    assert!(
        env.beq(input, &Env::new(), output, &Env::new()),
        "Expected terms to be equal under beta equality:\n\n------\n{}\n------ Reduces to -->\n{}\n------\n\n------\n{}\n------ Reduces to -->\n{}\n------\n\n.",
        env.index_to_string(input, false),
        env.index_to_string(input, true),
        env.index_to_string(output, false),
        env.index_to_string(output, true),
    );





    // let input.beta_reduce(input.root)

    // let env = TcEnv::new();
    // env.
    //
    // assert_eq!(
    //     .beta_reduce(input, &Env::new()).0,
    //     output
    // );
}

test_each_file! { for ["in", "out"] in "prism-compiler/programs/beta_reduce/" => test }


#[test]
fn placeholder() {

}