use crate::parser::parse_test;

parse_test! {
name: infinite_repeat
syntax: r#"
    rule start:
        X() <- ""* "x"

    "#

passing tests:

failing tests:
    ""
    "x"

}
