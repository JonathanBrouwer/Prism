use crate::parser::parse_test;
parse_test! {
name: list_ast
syntax: r#"
    ast Test:
        Leaf()
        Nodes(nodes: [Input])


    rule start -> Test:
        Nodes(ns) <- "(" ns:start* ")"
        Leaf() <- "x"

    "#
passing tests:
    "x" => "Leaf()"
    "()" => "Nodes([])"
    "(x)" => "Nodes([Leaf()])"
    "(xx)" => "Nodes([Leaf(), Leaf()])"
    "((x))" => "Nodes([Nodes([Leaf()])])"

failing tests:
    "xx"
    "(x"
    "x)"
    "(y)"
    "(x))"
    "((x)"
    ""
}

parse_test! {
name: list_rule
syntax: r#"
    ast Test:
        Leaf()
        Nodes(nodes: [Input])


    rule start -> [Test]:
        other*


    rule other -> Test:
        Nodes(ns) <- "(" ns:other* ")"
        Leaf() <- "x"

    "#
passing tests:
    "x" => "[Leaf()]"
    "()" => "[Nodes([])]"
    "(x)" => "[Nodes([Leaf()])]"
    "(xx)" => "[Nodes([Leaf(), Leaf()])]"
    "((x))" => "[Nodes([Nodes([Leaf()])])]"
    "xx" => "[Leaf(), Leaf()]"
    "" => "[]"

failing tests:
    "(x"
    "x)"
    "(y)"
    "(x))"
    "((x)"
}
