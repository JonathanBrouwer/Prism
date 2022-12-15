use crate::parser::parse_test;
parse_test! {
name: recovery1
syntax: r#"
rule start = "abcd"
"#
passing tests:
    "abcd" => "'abcd'"

failing tests:
    "" => "0..0"
    "a" => "1..1"
    "b" => "0..0 1..1"
    "abd" => "2..2"
    "abc" => "3..3"
    "ad" => "1..1"
    "ac" => "1..1 2..2"
    "axcd" => "1..2"
    "axyd" => "1..4"
    "xabcd" => "0..2"
}
