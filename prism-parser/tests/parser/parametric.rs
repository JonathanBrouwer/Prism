use crate::parser::parse_test;

parse_test! {
name: parametric_first_order
syntax: r##"
    rule start = id(hash);

    rule id(n) = n;

    rule hash = "x" <- "#";

    "##
passing tests:
    "#" => "'x'"
failing tests:
    "x"
}

parse_test! {
name: parametric_first_order_multiple
syntax: r##"
    rule start = "x" <- a_then_b(a, b);

    rule a() = "a";

    rule b() = "b";

    rule a_then_b(a, b) = a b;

    rule layout = " ";
    "##
passing tests:
    "ab" => "'x'"
    "a b" => "'x'"
failing tests:
    "a"
    "b"
}

parse_test! {
name: pass_through
syntax: r##"
    rule start = "x" <- a(hash);

    rule a(r) = b(r);

    rule b(r) = r;

    rule hash = "#";

    rule layout = " ";
    "##
passing tests:
    "#" => "'x'"
failing tests:
    "a"
}

parse_test! {
name: caching_test
syntax: r##"
    rule start = id(x) / id(y);

    rule id(r) = r;

    rule x = "x";
    rule y = "y";

    "##
passing tests:
    "x" => "'x'"
    "y" => "'y'"
failing tests:
    "z"
}
parse_test! {
name: pass_value_twice
syntax: r##"
    rule start = v <- r1:id(letter) r2:id(r1) v:r2;

    rule letter = ['a'-'z'];
    rule id(v) = $v;
    "##
passing tests:
    "x" => "'x'"

failing tests:
    ""
    "xy"
}

parse_test! {
name: pass_complex
syntax: r##"
    rule start = do(x <- x:"x" "y");
    rule do(v) = v;
    "##
passing tests:
    "xy" => "'x'"

failing tests:
    "x"
    "y"
    ""
    "xyz"
}

parse_test! {
name: parametric_literal
syntax: r##"
    rule start = id("hey");
    rule id(v) = v;
    "##
passing tests:
    "hey" => "'hey'"

failing tests:
    ""
    "heyy"
}

parse_test! {
name: parametric_ignore
syntax: r##"
    rule start = id("hey");
    rule id(_) = "hai";
    "##
passing tests:
    "hai" => "'hai'"

failing tests:
    "hey"
}

parse_test! {
name: parametric_higher_order
syntax: r##"
    rule start = map_x(id);
    rule map_x(f) = f("x");
    rule id(v) = v;
    "##
passing tests:
    "x" => "'x'"

failing tests:
    "y"
}

parse_test! {
name: simple_closure
syntax: r##"
    rule start = do(test("x"));
    rule do(f) = f;
    rule test(x) = Vals(a, "y") <- a:x "y";
    "##
passing tests:
    "xy" => "Vals('x', 'y')"

failing tests:
    "y"
    "x"
    ""
    "yx"
}

parse_test! {
name: val_param1
syntax: r##"
    rule start = do($X());
    rule do(f) = f <- "";
    "##
passing tests:
    "" => "X()"

failing tests:
    "x"
}

parse_test! {
name: parametric_this
syntax: r##"
    rule start = do($A());
    rule do(v) {
        v <- "x";
        w <- "y" w:#this($B(v));
    }
    "##
passing tests:
    "x" => "A()"
    "yx" => "B(A())"
    "yyx" => "B(B(A()))"

failing tests:
    "y"
    "xy"
    "yxy"
}

parse_test! {
name: parametric_next
syntax: r##"
    rule start = do($A());
    rule do(v) {
        group a {
            w <- "y" w:#next($B(v));
        }
        group b {
            v <- "x";
        }
    }
    "##
passing tests:
    "x" => "A()"
    "yx" => "B(A())"

failing tests:
    "y"
    "xy"
    "yxy"
    "yyx"
}

parse_test! {
name: parametric_this2
syntax: r##"
    rule start = do($A(), $C());
    rule do(v, w) {
        v <- "x";
        y <- "y" y:#this($B(v), w);
    }
    "##
passing tests:
    "x" => "A()"
    "yx" => "B(A())"
    "yyx" => "B(B(A()))"

failing tests:
    "y"
    "xy"
    "yxy"
}

//TODO simple currying
// parse_test! {
// name: curried
// syntax: r##"
//     rule start = do(test("x"));
//     rule do(f) = f("y");
//     rule test(x, y) = Vals(a, b) <- a:x b:y;
//     "##
// passing tests:
//     "xy" => "Vals('x', 'y')"
//
// failing tests:
//     "y"
//     "x"
//     ""
//     "yx"
// }
