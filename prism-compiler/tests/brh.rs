use test_each_file::test_each_file;
use prism_compiler::coc::env::Env;
use prism_compiler::coc::type_check::TcEnv;
use prism_compiler::parse_prism;
use prism_parser::parser::parser_instance::Arena;

fn test([input, output]: [&str; 2]) {
    let arena = Arena::new();
    let input = parse_prism(input, &arena).expect("Failed to parse input");
    let output = parse_prism(output, &arena).expect("Failed to parse output");
    
    let env = TcEnv::new();
    env.

    assert_eq!(
        .brh(input, &Env::new()).0,
        output
    );
}

test_each_file! { for ["in", "brh"] in "prism-compiler/programs/eval/" => test }