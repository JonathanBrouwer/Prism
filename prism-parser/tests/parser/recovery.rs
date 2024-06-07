use crate::parser::parse_test;

parse_test! {
name: recovery1
syntax: r#"
rule start = "seq" <- "a" "b" "c" "d";
"#
passing tests:
    "abcd" => "'seq'"

failing tests:
    "" => "0..0"
    "a" => "1..1"
    "b" => "0..0 1..1"
    "abd" => "2..2"
    "abc" => "3..3"
    "ad" => "1..1"
    "ac" => "1..1 2..2"
    "axcd" => "1..2"
    "axyd" => "1..3"
    "xabcd" => "0..2"
}

parse_test! {
name: recovery_with_norecovery
syntax: r#"
rule start = test*;

rule test = w <- w:#str(word) ";";

rule word {
    #[disable_recovery]
    ['a'-'z']+;
}
"#
passing tests:
    "aaaaa;a;aa;" => "['aaaaa', 'a', 'aa']"

failing tests:
    "a#a;aa;" => "1..2"
    "a#;aa;" => "1..2"
}

parse_test! {
name: recovery_new
syntax: r#"
rule start {
    (s <- "{" s:stmt "}")*;
}
rule stmt {
    "seq" <- "abc" ";";
}
rule layout = [' ' | '\n'];
"#
passing tests:
    "{abc;}{abc;}{abc;}" => "['seq', 'seq', 'seq']"

failing tests:
    "{abc}{abc;}{abc;}" => "4..4"
    "{abx;}{abc;}{abc;}" => "3..4"
    "{ab;}{abc;}{ac;}" => "3..3 13..14"
    "{abc}{abc;}{abc;}" => "4..4"
}
