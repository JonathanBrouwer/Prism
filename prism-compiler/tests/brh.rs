use prism_compiler::coc::env::Env;
use prism_compiler::coc::TcEnv;
use prism_compiler::parse_prism;
use prism_parser::parser::parser_instance::Arena;
use test_each_file::test_each_file;

fn test([input, output]: [&str; 2]) {
    let input = parse_prism(input).expect("Failed to parse input");
    let output = parse_prism(output).expect("Failed to parse output");

    // let env = TcEnv::new();
    // env.
    //
    // assert_eq!(
    //     .brh(input, &Env::new()).0,
    //     output
    // );
}

test_each_file! { for ["in", "brh"] in "prism-compiler/programs/eval/" => test }
