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
name: guid
syntax: r#"
    rule start = Guids(a, b, c) <- a:#guid b:#guid c:#guid;
    "#
passing tests:
    "" => "Guids(Guid(0), Guid(1), Guid(2))"

failing tests:
}
