use crate::parser::parse_test;
parse_test! {
name: sequence
syntax: r#"
    rule start -> Input:
        str("a" ['w'-'y'] "q")

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
    rule start -> Input:
        "a" / ['w'-'y'] / "q"

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
    ast Test:
        TestC(left: Input, right: Input)


    rule start -> Input:
        TestC(c, d) <- "a" c:['w'-'y'] d:"q"

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
