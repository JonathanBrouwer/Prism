use crate::parser::parse_test;
parse_test! {
name: sequence
syntax: r#"
    rule start = #str("a" ['w'-'y'] "q");

    "#
passing tests:
    "awq" => "'awq'"
    "axq" => "'axq'"
    "ayq" => "'ayq'"

failing tests:
    "a"
    "aw"
    "ax"
    "ay"
    "aqq"
    "aaq"
    "bwq"
    ""
    "awqq"
}

parse_test! {
name: choice
syntax: r#"
    rule start = "a" / ['w'-'y'] / "q";

    "#
passing tests:
    "a" => "'a'"
    "w" => "'w'"
    "y" => "'y'"
    "q" => "'q'"

failing tests:
    "aw"
    ""
    "b"
    "z"
    "wy"
    "wq"
}

parse_test! {
name: action
syntax: r#"
    rule start = TestC(c, d) <- "a" c:['w'-'y'] d:"q";

    "#
passing tests:
    "awq" => "TestC('w', 'q')"
    "axq" => "TestC('x', 'q')"
    "ayq" => "TestC('y', 'q')"

failing tests:
    "a"
    "aw"
    "ax"
    "ay"
    "aqq"
    "aaq"
    "bwq"
    ""
    "awqq"
}

parse_test! {
name: use_thing_twice
syntax: r#"
    rule start = Test(a, a) <- a:"q";
    "#
passing tests:
    "q" => "Test('q', 'q')"

failing tests:
}
