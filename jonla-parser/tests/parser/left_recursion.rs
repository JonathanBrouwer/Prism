use crate::parser::parse_test;

parse_test! {
name: left_recursion
syntax: r#"
    rule start:
        X(e) <- e:@this "X"
        --
        Y() <- "Y"

    "#
passing tests:
    "Y" => "Y()"
    "YX" => "X(Y())"
    "YXX" => "X(X(Y()))"

failing tests:
    "XY"
    "X"

}

parse_test! {
name: left_recursion_direct
syntax: r#"
    rule start:
        X(e) <- e:start "X"
        --
        Y() <- "Y"

    "#
passing tests:
    "Y" => "Y()"
    "YX" => "X(Y())"
    "YXX" => "X(X(Y()))"

failing tests:
    "XY"
    "X"

}

