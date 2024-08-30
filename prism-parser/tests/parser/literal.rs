use crate::parser::parse_test;

parse_test! {
name: empty
syntax: r#"
    rule start = "";
    "#
passing tests:
    "" => "''"
failing tests:
    "x"
}

parse_test! {
name: literal
syntax: r#"
    rule start = "lol";
   
    "#
passing tests:
    "lol" => "'lol'"
failing tests:
    "lolz"
    "loll"
    "lol "
    ""
    "l"
    "lo"
    " lol"
    "lo\nn"
}

parse_test! {
name: literal_indirect
syntax: r#"
    rule start = lol;

    rule lol = "lol";

    "#
passing tests:
    "lol" => "'lol'"
failing tests:
    "lolz"
    "loll"
    "lol "
    ""
    "l"
    "lo"
    " lol"
    "lo\nn"
}

parse_test! {
name: charclass
syntax: r#"
    rule start = #str([ 'w'-'z' | '8' | 'p'-'q' ]);

    "#
passing tests:
    "8" => "'8'"
    "w" => "'w'"
    "x" => "'x'"
    "y" => "'y'"
    "z" => "'z'"
    "p" => "'p'"
    "q" => "'q'"

failing tests:
    "a"
    "b"
    "v"
    "7"
    "9"
    "o"
    "r"
    " "
    "w8"
    "8w"
}
