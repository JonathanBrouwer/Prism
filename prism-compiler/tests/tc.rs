use test_each_file::test_each_file;
use prism_compiler::coc::env::Env;
use prism_compiler::coc::type_check::TcEnv;
use prism_compiler::parse_prism;
use prism_parser::parser::parser_instance::Arena;

fn test([input]: [&str; 1]) {
    let arena = Arena::new();
    let input = parse_prism(input, &arena).expect("Failed to parse input");

    TcEnv::new().tc_expr(input, &Env::new());
}

#[test]
fn placeholder() {

}

test_each_file! { for ["in"] in "prism-compiler/programs/tc/" => test }