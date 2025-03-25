use bumpalo::Bump;
use prism_compiler::lang::PrismEnv;
use prism_parser::core::allocs::Allocs;
use test_each_file::test_each_file;

fn test_ok([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input_str, rest) = rest.split_once("### Eval\n").unwrap();
    let (_eval, _expected_typ) = rest.split_once("### Type\n").unwrap();

    let bump = Bump::new();
    let mut env = PrismEnv::new(Allocs::new(&bump));
    let input = env.load_test(input_str, "input");
    let _input = env.parse_file(input);

    env.assert_no_errors();
}
test_each_file! { for ["test"] in "prism-compiler/programs/ok" => test_ok }

#[test]
fn placeholder() {}
