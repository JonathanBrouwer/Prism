use crate::parser::parse_test;

parse_test! {
name: pos_lookahead
syntax: r#"
rule start -> Str = s <- pos("x") s:['x'|'y']
"#
passing tests:
    "x" => "'x'"
failing tests:
    "y"
}

parse_test! {
name: neg_lookahead
syntax: r#"
rule start -> Str = s <- neg("x") s:['x'|'y']
"#
passing tests:
    "y" => "'y'"
failing tests:
    "x"
}