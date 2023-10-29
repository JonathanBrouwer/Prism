use crate::parser::parse_test;

parse_test! {
name: adaptive
syntax: r#"
    rule start = block
    rule block:
        b <- "grammar" "{" g:grammar(prule_action) "}" ";" b:@adapt(g, block)
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
    "### => "[Let(Construct('Z', []))]"
    // Add to base redundant specification
    r###"
    grammar {
        rule expr:
            -- x
            -- additive
            -- base
            Z() <- "z"
            -- y
    };
    let z;
    "### => "[Let(Construct('Z', []))]"
    // Add minus
    r###"
    grammar {
        rule expr:
            -- additive
            Sub(x, y) <- x:@next "-" y:@this
    };
    let x + y - x + x + y - x;
    "### => "[Let(Add(X(), Construct('Sub', [Name('x'), Name('y')])))]"
    // Add mul + minus
    r###"
    grammar {
        rule expr:
            -- additive
            Sub(x, y) <- x:@next "-" y:@this
            -- multiplicative
            Mul(x, y) <- x:@next "*" y:@this
            -- base
    };
    let x + y * y - x * x + y * x;
    "### => "[Let(Add(X(), Construct('Sub', [Name('x'), Name('y')])))]"
    // Add mul + minus seperately (1)
    r###"
    grammar {
        rule expr:
            -- additive
            -- multiplicative
            Mul(x, y) <- x:@next "*" y:@this
            -- base
    };
    grammar {
        rule expr:
            -- additive
            Sub(x, y) <- x:@next "-" y:@this
    };
    let x + y * y - x * x + y * x;
    "### => "[Let(Add(X(), Construct('Sub', [Name('x'), Name('y')])))]"
    // Add mul + minus seperately (2)
    r###"
    grammar {
        rule expr:
            -- additive
            Sub(x, y) <- x:@next "-" y:@this
    };
    grammar {
        rule expr:
            -- additive
            -- multiplicative
            Mul(x, y) <- x:@next "*" y:@this
            -- base
    };
    let x + y * y - x * x + y * x;
    "### => "[Let(Add(X(), Construct('Sub', [Name('x'), Name('y')])))]"

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

parse_test! {
name: adaptive_simple
syntax: r#"
    rule start:
        b <- "{" g:grammar(prule_action) "}" b:@adapt(g, start)
        X() <- "x"
    "#
passing tests:

failing tests:
    r###"y"###
}

parse_test! {
name: adaptive_sub
syntax: r#"
    rule start:
        b <- "{" g:grammar(prule_action) "}" b:(start / @adapt(g, start))
        X() <- "x"
        sub

    rule sub:
        Z() <- "z"

    rule layout = [' ' | '\n']
    "#
passing tests:
    r###"
    {
        rule sub:
            Y() <- "y"
    }
    x
    "### => "X()"
    r###"
    {
        rule sub:
            Y() <- "y"
    }
    y
    "### => "Construct('Y', [])"
    r###"
    {
        rule sub:
            Y() <- "y"
    }
    z
    "### => "Z()"

failing tests:
    r###"
    {
        rule sub:
            Y() <- "y"
    }
    w
    "###

}
