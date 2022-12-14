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

parse_test! {
name: infinite_repeat_delim
syntax: r#"
    rule start:
        X() <- @repeat("", ",", *) "x"

    "#

passing tests:
    "x" => "X()"
    ",x" => "X()"
    ",,x" => "X()"

failing tests:
    ""

}

