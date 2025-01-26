use bumpalo::Bump;
use prism_compiler::lang::PrismEnv;
use prism_compiler::parser::parse_prism_in_env;
use prism_parser::core::cache::Allocs;
use prism_parser::error::aggregate_error::ParseResultExt;
use test_each_file::test_each_file;

fn test_ok([test]: [&str; 1]) {
    let (_, rest) = test.split_once("### Input\n").unwrap();
    let (input_str, rest) = rest.split_once("### Eval\n").unwrap();
    let (_eval, _expected_typ) = rest.split_once("### Type\n").unwrap();

    let bump = Bump::new();
    let mut env = PrismEnv::new(Allocs::new(&bump));
    let _input = parse_prism_in_env(input_str, &mut env).unwrap_or_eprint();
}
test_each_file! { for ["test"] in "prism-compiler/programs/ok" => test_ok }
