use crate::parser::parse_test;

parse_test! {
name: arithmetic
syntax: r#"
rule start = block;
rule block {
    b <- "grammar" "{" g:grammar(expr) "}" ";" b:#adapt(g, block);
    expr(#env);
}
rule expr(_) {
    group additive {
        Add(x, y) <- x:#next "+" y:#this;
    }
    group multiplicative {
        Mul(x, y) <- x:#next "*" y:#this;
    }
    group base {
        Block(b) <- "(" b:block ")";
        UnaryMinus(v) <- "-" v:#this;
        Num(n) <- n:#str(['0'-'9']*);
    }
}

rule layout = [' ' | '\n'];
    "#
passing tests:
    // Simple
    r###"
    1 * 2 + -3
    "### => "Add(Mul(Num('1'), Num('2')), UnaryMinus(Num('3')))"
    // Add binary minus
    //TODO when grammar blocks can capture context, make this test nicer
    r###"
    grammar {
        rule expr {
            group additive {
                1 + (-2) <- x:#next "-" y:#this;
            }
        }
    };
    1 - 2
    "### => "Add(Num('1'), Block(UnaryMinus(Num('2'))))"

failing tests:
}
