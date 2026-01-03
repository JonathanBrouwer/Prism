use crate::parser::parse_test;

parse_test! {
name: infinite_repeat
syntax: r#"
    rule start = X() <- ""* "x";
"#

passing tests:

failing tests:
    ""
    "x"

}

parse_test! {
name: infinite_repeat_delim
syntax: r#"
    rule start = X() <- #repeat("", ",", *) "x";

    "#

passing tests:
    "x" => "X()"
    ",x" => "X()"
    ",,x" => "X()"

failing tests:
    ""

}

parse_test! {
name: infinite_directrec
syntax: r#"
    rule start = X() <- start;

    "#

passing tests:

failing tests:
    ""
    "x"

}

parse_test! {
name: infinite_mutualrec
syntax: r#"
    rule start = X() <- other;
    rule other = X() <- start;

    "#

passing tests:

failing tests:
    ""
    "x"

}

parse_test! {
name: infinite_emptyrec
syntax: r#"
    rule start = X() <- "" other;
    rule other = X() <- "" start;

    "#

passing tests:

failing tests:
    ""
    "x"

}

parse_test! {
name: infinite_test
syntax: r#"
    rule layout = " ";

    rule num {
        #[token("number")]
        #str("x"+);
    }

    rule start {
        Num(e) <- e:num;
    }

    "#
passing tests:

failing tests:
    "x x"
}
