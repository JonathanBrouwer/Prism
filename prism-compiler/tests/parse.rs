use prism_compiler::lang::env::Env;
use prism_compiler::lang::error::TypeResultExt;
use prism_compiler::lang::TcEnv;
use prism_compiler::parser::parse_prism_in_env;
use prism_parser::error::aggregate_error::ParseResultExt;
use test_each_file::test_each_file;

fn test_ok([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input_str, rest) = rest.split_once("### Eval\n").unwrap();
    let (_eval, expected_typ) = rest.split_once("### Type\n").unwrap();

    let mut env = TcEnv::default();
    let input = parse_prism_in_env(input_str, &mut env).unwrap_or_eprint();
}
test_each_file! { for ["test"] in "prism-compiler/programs/ok" => test_ok }

#[test]
fn benchmark() {
    const REPEAT_TIMES: usize = 1;
    let program = include_str!("../programs/ok/church_bools_and_or.test");
    let (_, rest) = program.split_once("### Input\n").unwrap();
    let (input_str, rest) = rest.split_once("### Eval\n").unwrap();
    let (_eval, _expected_typ) = rest.split_once("### Type\n").unwrap();

    for _ in 0..REPEAT_TIMES {
        let mut env = TcEnv::default();
        parse_prism_in_env(input_str, &mut env).unwrap_or_eprint();
    }
}
