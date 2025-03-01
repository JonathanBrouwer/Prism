use crate::parser::parse_test;

parse_test! {
name: adaptive
syntax: r#"
    rule start = block;
    rule block {
        b <- "grammar" "{" g:grammar(prule_action) "}" ";" b:#adapt(GrammarFile,  g, block);
        s .. b <- s:stmt ";" b:block;
        [] <- "";
    }

    rule stmt {
        Let(e) <- "let" e:expr;
        Do() <- "do";
    }

    rule expr {
        group additive {
            Add(x, y) <- x:#next "+" y:#this;
        }
        group base {
            Block(b) <- "{" b:block "}";
            X() <- "x";
            Y() <- "y";
        }
    }

    rule layout = [' ' | '\n'];
    "#
passing tests:
    // Simple, add to base
    r###"
    grammar {
        adapt rule expr {
            adapt group base {
                Z() <- "z";
            }
        }
    };
    let z;
    "### => r#"[Let(Z())]"#
    // Add to base redundant specification
    r###"
    grammar {
        adapt rule expr {
            group x {}
            adapt group additive {}
            adapt group base {
                Z() <- "z";
            }
            group y {}
        }
    };
    let z;
    "### => r#"[Let(Z())]"#
    // Add minus
    r###"
    grammar {
        adapt rule expr {
            adapt group additive {
                Sub(x, y) <- x:#next "-" y:#this;
            }
        }
    };
    let x + y - x + x + y - x;
    "### => r#"[Let(Add(X(), Sub(Y(), Add(X(), Add(X(), Sub(Y(), X()))))))]"#
    // Add mul + minus
    r###"
    grammar {
        adapt rule expr {
            adapt group additive {
                Sub(x, y) <- x:#next "-" y:#this ;
            }
            group multiplicative {
                Mul(x, y) <- x:#next "*" y:#this;
            }
            adapt group base {}
        }
    };
    let x + y * y - x * x + y * x;
    "### => r#"[Let(Add(X(), Sub(Mul(Y(), Y()), Add(Mul(X(), X()), Mul(Y(), X())))))]"#
    // Add mul + minus seperately (1)
    r###"
    grammar {
        adapt rule expr {
            adapt group additive {}
            group multiplicative {
                Mul(x, y) <- x:#next "*" y:#this;
            }
            adapt group base {}
        }
    };
    grammar {
        adapt rule expr {
            adapt group additive {
                Sub(x, y) <- x:#next "-" y:#this;
            }
        }
    };
    let x + y * y - x * x + y * x;
    "### => r#"[Let(Add(X(), Sub(Mul(Y(), Y()), Add(Mul(X(), X()), Mul(Y(), X())))))]"#
    // Add mul + minus seperately (2)
    r###"
    grammar {
        adapt rule expr{
            adapt group additive {
                Sub(x, y) <- x:#next "-" y:#this;
            }
        }
    };
    grammar {
        adapt rule expr {
            adapt group additive {}
            group multiplicative {
                Mul(x, y) <- x:#next "*" y:#this;
            }
            adapt group base {}
        }
    };
    let x + y * y - x * x + y * x;
    "### => r#"[Let(Add(X(), Sub(Mul(Y(), Y()), Add(Mul(X(), X()), Mul(Y(), X())))))]"#

failing tests:
    // Turns order around
    r###"
    grammar {
        adapt rule expr {
            adapt group base {
                Z() <- "z";
            }
            adapt group additive {}
        }
    };
    let z;
    "###
}

parse_test! {
name: adaptive_simple
syntax: r#"
    rule start {
        b <- "{" g:grammar(prule_action) "}" b:#adapt(GrammarFile,  g, start);
        X() <- "x";
    }
    "#
passing tests:

failing tests:
    r###"y"###
}

parse_test! {
name: adaptive_sub
syntax: r#"
    rule start {
        b <- "{" g:grammar(prule_action) "}" b:<start / #adapt(GrammarFile,  g, start)>;
        X() <- "x";
        sub;
    }

    rule sub {
        Z() <- "z";
    }

    rule layout = [' ' | '\n'];
    "#
passing tests:
    r###"
    {
        adapt rule sub {
            adapt group {
                Y() <- "y";
            }
        }
    }
    x
    "### => "X()"
    r###"
    {
        adapt rule sub {
            adapt group {
                Y() <- "y";
            }
        }
    }
    y
    "### => r#"Y()"#
    r###"
    {
        adapt rule sub {
            adapt group {
                Y() <- "y";
            }
        }
    }
    z
    "### => "Z()"

failing tests:
    r###"
    {
        adapt rule sub {
            Y() <- "y";
        }
    }
    w
    "###

}

parse_test! {
name: adaptive_sub2
syntax: r#"
    rule start {
        b <- "{" g:grammar(prule_action) "}" b:<sub2 / #adapt(GrammarFile,  g, sub2)>;
    }

    rule sub2 = sub;

    rule sub = Z() <- "z";"#
passing tests:
    r###"{
    adapt rule sub {
        Y() <- "y";
    }}y"### => r#"Y()"#

failing tests:

}

parse_test! {
name: same_name_rule
syntax: r#"
    rule start {
        b <- "{" g:grammar(prule_action) "}" b:#adapt(GrammarFile,  g, start);
        X() <- "x";
    }"#
passing tests:
    r###"x"### => "X()"

    r###"{rule start {
        Y() <- "y";
    }}x"### => "X()"
failing tests:
    r###"{rule start {
        Y() <- "y";
    }}y"###
}
