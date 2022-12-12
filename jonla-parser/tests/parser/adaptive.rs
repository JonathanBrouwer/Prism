use crate::parser::parse_test;
parse_test! {
name: adaptive
syntax: r#"
    rule start = block
    rule block:
        b <- "grammar" "{" g:@grammar "}" ";" b:@adapt(g, block)
        s :: b <- s:stmt ";" b:block
        [] <- ""

    rule stmt:
        Let(e) <- "let" e:expr
        Do() <- "do"

    rule expr:
        -- additive
        Add(x, y) <- x:@next "+" y:@this
        -- base
        Block(b) <- "{" b:block "}"
        X() <- "x"
        Y() <- "y"

    rule layout = [' ' | '\n']
    "#
passing tests:
    // Simple, add to base
    r###"
    grammar {
        rule expr:
            -- base
            Z() <- "z"
    };
    let z;
    "### => "[Let(Z())]"
    // Add to base redundant specification
    r###"
    grammar {
        rule expr:
            -- additive
            -- base
            Z() <- "z"
    };
    let z;
    "### => "[Let(Z())]"

failing tests:
    // Turns order around
    r###"
    grammar {
        rule expr:
            -- base
            Z() <- "z"
            -- additive
    };
    let z;
    "###
}
