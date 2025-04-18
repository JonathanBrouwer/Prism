use prism_compiler::lang::PrismDb;
use std::fs;
use std::path::Path;
use test_each_file::test_each_path;

fn test_ok([test_path]: [&Path; 1]) {
    let test = fs::read_to_string(test_path).unwrap();

    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input_str, rest) = rest.split_once("### Eval\n").unwrap();
    let (_eval, _expected_typ) = rest.split_once("### Type\n").unwrap();

    let mut env = PrismDb::new();
    let input = env.load_test(input_str, "input");
    let _input = env.parse_prism_file(input);

    env.assert_no_errors();
}
test_each_path! { for ["test"] in "prism-compiler/programs/ok" => test_ok }

#[test]
fn placeholder() {}
