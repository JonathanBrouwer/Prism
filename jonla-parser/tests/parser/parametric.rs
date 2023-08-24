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

parse_test! {
name: parametric_first_order_multiple
syntax: r##"
    rule start:
        "x" <- a_then_b(a, b)

    rule a():
        "a"

    rule b():
        "b"

    rule a_then_b(a, b):
        a b

    rule layout = " "
    "##
passing tests:
    "ab" => "'x'"
    "a b" => "'x'"
failing tests:
    "a"
    "b"
}

parse_test! {
name: pass_through
syntax: r##"
    rule start:
        "x" <- a(hash)

    rule a(r):
        b(r)

    rule b(r):
        r

    rule hash = "#"

    rule layout = " "
    "##
passing tests:
    "#" => "'x'"
failing tests:
    "a"
}
