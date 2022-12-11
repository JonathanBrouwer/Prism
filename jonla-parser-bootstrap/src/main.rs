use std::fs::File;
use std::process::exit;
use jonla_parser::grammar;
use jonla_parser::grammar::GrammarFile;
use jonla_parser::parser::actual::error_printer::print_tree_error;
use jonla_parser::parser::actual::parser_rule::run_parser_rule;
use jonla_parser::parser::core::stream::StringStream;
use crate::from_action_result::parse_grammarfile;

mod from_action_result;

fn main() {
    // Get old grammar to parse new grammar
    let grammar: GrammarFile =
        match grammar::grammar_def::toplevel(include_str!("../../jonla-parser-bootstrap/resources/meta.grammar")) {
            Ok(ok) => ok,
            Err(err) => {
                panic!("{}", err);
            }
        };

    let input = include_str!("../../jonla-parser-bootstrap/resources/meta.grammar");
    let input_stream: StringStream = input.into();
    let result: Result<_, _> = run_parser_rule(&grammar, "toplevel", input_stream);

    match result {
        Ok(o) => {
            let grammar2 = parse_grammarfile(&*o.1, input);
            assert_eq!(grammar, grammar2); // Safety check

            let mut file = File::create("jonla-parser-bootstrap/resources/bootstrap.json").unwrap();
            serde_json::to_writer_pretty(&mut file, &grammar2).unwrap();
        }
        Err(e) => {
            print_tree_error(e, "file", input, true);
            exit(1);
        },
    }
}
