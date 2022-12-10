use crate::parser::parse_test;
parse_test! {
name: repeat_star
syntax: r#"
    rule start:
        str([ 'w'-'z' | '8' | 'p'-'q' ]*)

    "#
passing tests:
    "8" => "'8'"
    "w" => "'w'"
    "x" => "'x'"
    "y" => "'y'"
    "z" => "'z'"
    "p" => "'p'"
    "q" => "'q'"
    "" => "''"
    "8w"  => "'8w'"
    "w8" => "'w8'"
    "wxyz8pqpq8wz" => "'wxyz8pqpq8wz'"

failing tests:
    "a"
    "b"
    "v"
    "7"
    "9"
    "o"
    "r"
    " "
    "wxya"
    "w8 "
}

parse_test! {
name: repeat_plus
syntax: r#"
    rule start:
        str([ 'w'-'z' | '8' | 'p'-'q' ]+)

    "#
passing tests:
    "8" => "'8'"
    "w" => "'w'"
    "x" => "'x'"
    "y" => "'y'"
    "z" => "'z'"
    "p" => "'p'"
    "q" => "'q'"
    "8w"  => "'8w'"
    "w8" => "'w8'"
    "wxyz8pqpq8wz" => "'wxyz8pqpq8wz'"

failing tests:
    "a"
    "b"
    "v"
    "7"
    "9"
    "o"
    "r"
    " "
    "wxya"
    "w8 "
    ""
}

parse_test! {
name: repeat_option
syntax: r#"
    rule start:
        str([ 'w'-'z' | '8' | 'p'-'q' ]?)

    "#
passing tests:
    "8" => "'8'"
    "w" => "'w'"
    "x" => "'x'"
    "y" => "'y'"
    "z" => "'z'"
    "p" => "'p'"
    "q" => "'q'"
    "" => "''"

failing tests:
    "a"
    "b"
    "v"
    "7"
    "9"
    "o"
    "r"
    " "
    "wxya"
    "w8 "
    "8w"
    "w8"
    "wxyz8pqpq8wz"
}
