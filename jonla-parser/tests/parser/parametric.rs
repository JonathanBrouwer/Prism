use crate::parser::parse_test;

parse_test! {
name: parametric
syntax: r##"
    rule start:
        hash("x")

    rule hash(n):
        n <- "#"

    "##
passing tests:
    "#" => "'x'"
failing tests:
    "x"
}

parse_test! {
name: parametric_first_order
syntax: r##"
    rule start:
        id(hash)

    rule id(n):
        n

    rule hash:
        "x" <- "#"

    "##
passing tests:
    "#" => "'x'"
failing tests:
    "x"
}