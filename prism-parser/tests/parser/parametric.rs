use crate::parser::parse_test;

parse_test! {
name: parametric_first_order
syntax: r##"
    rule start = id(hash);

    rule id(n) = n;

    rule hash = "x" <- "#";

    "##
passing tests:
    "#" => "'x'"
failing tests:
    "x"
}

parse_test! {
name: parametric_first_order_multiple
syntax: r##"
    rule start = "x" <- a_then_b(a, b);

    rule a() = "a";

    rule b() = "b";

    rule a_then_b(a, b) = a b;

    rule layout = " ";
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
    rule start = "x" <- a(hash);

    rule a(r) = b(r);

    rule b(r) = r;

    rule hash = "#";

    rule layout = " ";
    "##
passing tests:
    "#" => "'x'"
failing tests:
    "a"
}

parse_test! {
name: caching_test
syntax: r##"
    rule start = id(x) / id(y);

    rule id(r) = r;

    rule x = "x";
    rule y = "y";

    "##
passing tests:
    "x" => "'x'"
    "y" => "'y'"
failing tests:
    "z"
}

parse_test! {
name: pass_value
syntax: r##"
    rule start = Letters(v1, v2) <- v1:letter v2:id(v1);

    rule letter = ['a'-'z'];
    rule id(v) = v <- "";
    "##
passing tests:
    "x" => "Letters('x', 'x')"

failing tests:
    ""
    "xy"
}

parse_test! {
name: pass_value_twice
syntax: r##"
    rule start = id(id(letter));

    rule letter = ['a'-'z'];
    rule id(v) = v <- "";
    "##
passing tests:
    "x" => "'x'"

failing tests:
    ""
    "xy"
}
