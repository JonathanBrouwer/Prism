use crate::parser::parse_test;

parse_test! {
name: arith_layout
syntax: r#"
    rule layout = " ";

    rule num {
        #[token("number")]
        #str(['0'-'9']+);
    }

    rule start = e <- e:expr;


    rule expr {
        Add(l, r) <- l:expr2 "+" r:expr;
        Sub(l, r) <- l:expr2 "-" r:expr;
        expr2;
    }


    rule expr2 {
        Mul(l, r) <- l:expr3 "*" r:expr2;
        Div(l, r) <- l:expr3 "/" r:expr2;
        expr3;
    }

    rule expr3{
        Pow(l, r) <- l:expr3 "^" r:expr4;
        expr4;
    }

    rule expr4{
        Neg(e) <- "-" e:expr4;
        Num(e) <- e:num;
    }
    "#
passing tests:
    "123" => "Num('123')"
    "5 * 4 + 20 * 4 - 50" => "Add(Mul(Num('5'), Num('4')), Sub(Mul(Num('20'), Num('4')), Num('50')))"
    "5 * 4 - 20 * 4 + 50" => "Sub(Mul(Num('5'), Num('4')), Add(Mul(Num('20'), Num('4')), Num('50')))"
    "-5 * -4 - -20 * -4 + -50" => "Sub(Mul(Neg(Num('5')), Neg(Num('4'))), Add(Mul(Neg(Num('20')), Neg(Num('4'))), Neg(Num('50'))))"
    "1 + 2 * 3" => "Add(Num('1'), Mul(Num('2'), Num('3')))"
    "1 * 2 + 3" => "Add(Mul(Num('1'), Num('2')), Num('3'))"
    "1 - 2 / 3" => "Sub(Num('1'), Div(Num('2'), Num('3')))"
    "1 / 2 - 3" => "Sub(Div(Num('1'), Num('2')), Num('3'))"
    "-8" => "Neg(Num('8'))"

failing tests:
    ""
    "1+"
    "+1"
}

parse_test! {
name: num_layout
syntax: r#"
    rule layout = " ";

    rule num {
        #[token("number")]
        #str(['0'-'9']+);
    }

    rule start {
        Neg(e) <- "-" e:start;
        Num(e) <- e:num;
    }
    "#
passing tests:
    "123" => "Num('123')"
    "123 " => "Num('123')"
    "- 8" => "Neg(Num('8'))"

failing tests:
}

parse_test! {
name: trailing_layout
syntax: r#"
    rule layout = " ";
    rule start = "x";
    "#
passing tests:
    "x" => "'x'"
    "x " => "'x'"

failing tests:
}

parse_test! {
name: layout_in_rule
syntax: r#"
    rule layout = " ";

    rule start = Ok() <- " " "x" " " "y" "z" " ";

    "#
passing tests:
    " x y z " => "Ok()"
    " x yz " => "Ok()"
    "  x  y  z  " => "Ok()"
    "  x  yz  " => "Ok()"

failing tests:
    "xyz"
    "xyz "
    "xy z"
    "xy z "
    "x yz"
    "x yz "
    "x y z"
    "x y z "
    " xyz"
    " xyz "
    " xy z"
    " xy z "
    " x yz"
    " x y z"
}

parse_test! {
name: slice_layout
syntax: r#"
    rule layout = " ";

    rule start = #str("hey");

    "#
passing tests:
    "hey" => "'hey'"
    " hey" => "'hey'"
    "hey " => "'hey'"
    " hey " => "'hey'"

failing tests:
}

parse_test! {
name: slice_layout_2
syntax: r#"
    rule layout = " ";

    rule start = s <- "hi" s:#str("hey");

    "#
passing tests:
    "hihey" => "'hey'"
    "hi hey" => "'hey'"
    "hihey " => "'hey'"
    "hi hey " => "'hey'"
    "hi hey" => "'hey'"
    "hi  hey" => "'hey'"
    "hi hey " => "'hey'"
    "hi  hey " => "'hey'"

failing tests:
}

parse_test! {
name: slice_layout_3
syntax: r#"
    rule layout = " ";

    rule start = #str("x"*);

    "#
passing tests:
    "xxx" => "'xxx'"
    " xxx" => "'xxx'"
    "x x x" => "'x x x'"
    " x  x x" => "'x  x x'"
    " xx  x " => "'xx  x'"
    "x xx" => "'x xx'"
    " x" => "'x'"

failing tests:
}

parse_test! {
name: slice_layout_4
syntax: r#"
    rule layout = " ";

    rule start = s <- #pos(" ") s:#str("x"*);

    "#
passing tests:

    " xxx" => "'xxx'"
    " x  x x" => "'x  x x'"
    " xx  x " => "'xx  x'"
    " x" => "'x'"

failing tests:
    "xxx"
    "x x x"
    "x xx"
}

parse_test! {
name: slice_layout_5
syntax: r#"
    rule layout = " ";

    rule start = s <- #pos(" ") s:#str("x"*);

    "#
passing tests:
    " xxx" => "'xxx'"
    " x  x x" => "'x  x x'"
    " xx  x " => "'xx  x'"
    " x" => "'x'"

failing tests:
    "xxx"
    "x x x"
    "x xx"
}
