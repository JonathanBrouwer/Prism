fn main() {
    // let grammar: GrammarFile =
    //     match grammar::grammar_def::toplevel(include_str!("../../jonla-parser-bootstrap/resources/meta.grammar")) {
    //         Ok(ok) => ok,
    //         Err(err) => {
    //             panic!("{}", err);
    //         }
    //     };

    // let input = include_str!("../../jonla-parser-bootstrap/resources/meta.grammar");
    // let input_stream: StringStream = input.into();
    // let result: Result<_, _> = run_parser_rule(&grammar, "toplevel", input_stream);
    //
    // match result {
    //     Ok(o) => {
    //         let grammar2 = parse_grammarfile(&*o.1, input);
    //         assert_eq!(grammar, grammar2);
    //         println!("{:?}", grammar2)
    //     }
    //     Err(e) => print_tree_error(e, "file", input, true),
    // }

    // let filename = "program.jnl";
    // let input = include_str!("../resources/program.jnl");
    // let input_stream: StringStream = input.into();
    // let result: Result<_, _> = run_parser_rule(&grammar, "block", input_stream);
    //
    // match result {
    //     Ok(o) => println!("Result: {:?}", o.1.to_string(input)),
    //     // Err(e) => print_set_error(e, filename, input, false),
    //     Err(e) => print_tree_error(e, filename, input, true),
    // }
}
