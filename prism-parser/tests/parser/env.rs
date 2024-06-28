use crate::parser::parse_test;

parse_test! {
name: simple
syntax: r#"
rule start = #env;

"#
passing tests:
    "" => "Env(...)"

failing tests:
    "x"
}
