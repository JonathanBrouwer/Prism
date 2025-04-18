use crate::parser::parse_test;

parse_test! {
name: arithmetic
syntax: r#"
rule start = block;
rule block {
    b <- "grammar" "{" g:grammar(wrapped_expr) "}" ";" b:#adapt(GrammarFile,  g, block);
    expr;
}
rule wrapped_expr = RuleAction::Value("ActionResult", v) <- v:expr;

rule expr {
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
    r###"
    grammar {
        adapt rule expr {
            adapt group additive {
                1 + (-2) <- x:#next "-" y:#this;
            }
        }
    };
    1 - 2
    "### => "EnvCapture(Add(Num('1'), Block(UnaryMinus(Num('2')))), [VARS])"

failing tests:
}
