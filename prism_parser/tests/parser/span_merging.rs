parse_test! {
name: span_merging
syntax: r#"
rule start = #str(empty "x");

rule empty = "";

rule layout = [' '];
    "#
passing tests:
"x " => "'x'"
"x" => "'x'"
" x" => "'x'"

failing tests:
}
