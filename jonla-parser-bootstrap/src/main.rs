use jonla_parser::from_action_result::parse_grammarfile;
use jonla_parser::grammar::GrammarFile;
use jonla_parser::parser::actual::error_printer::print_set_error;
use jonla_parser::parser::actual::parser_rule::run_parser_rule;
use jonla_parser::parser::core::stream::StringStream;
use jonla_parser::META_GRAMMAR;
use std::fs::File;
use std::process::exit;

pub fn get_new_grammar(input: &str) -> GrammarFile {
    let input_stream: StringStream = StringStream::new(input);
    let result: Result<_, _> = run_parser_rule(&META_GRAMMAR, "toplevel", input_stream);

    match result {
        Ok(o) => parse_grammarfile(&o.1, input),
        Err(e) => {
            // print_tree_error(e, "file", input, true);
            print_set_error(e, "file", input, true);
            exit(1);
        }
    }
}

fn main() {
    let grammar: &'static GrammarFile = &META_GRAMMAR;

    let input = include_str!("../resources/meta.grammar");
    let grammar2 = get_new_grammar(input);

    assert_eq!(grammar, &grammar2); // Safety check

    let mut file = File::create("jonla-parser/resources/bootstrap.json").unwrap();
    serde_json::to_writer_pretty(&mut file, &grammar2).unwrap();
}
