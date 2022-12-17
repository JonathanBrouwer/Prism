#![allow(dead_code)]

use jonla_parser::grammar::from_action_result::parse_grammarfile;
use jonla_parser::grammar::grammar::GrammarFile;
use jonla_parser::error::error_printer::print_set_error;
use jonla_parser::core::stream::StringStream;
use jonla_parser::grammar::run::run_parser_rule;
use jonla_parser::META_GRAMMAR;
use std::fs::{File};
use std::process::exit;

pub fn get_new_grammar(input: &str) -> GrammarFile {
    let input_stream: StringStream = StringStream::new(input);
    let result: Result<_, _> = run_parser_rule(&META_GRAMMAR, "toplevel", input_stream);

    match result {
        Ok(o) => parse_grammarfile(&o.1, input),
        Err(es) => {
            for e in es {
                // print_tree_error(e, "file", input, true);
                print_set_error(e, "file", input, true);
            }
            exit(1);
        }
    }
}

fn main() {
    normal();
    // part1();
    // part2();
}

fn normal() {
    let input = include_str!("../resources/meta.grammar");
    let grammar2 = get_new_grammar(input);

    // let grammar: &'static GrammarFile = &META_GRAMMAR;
    // assert_eq!(grammar, &grammar2); // Safety check

    let mut file = File::create("jonla-parser/resources/bootstrap.json").unwrap();
    serde_json::to_writer_pretty(&mut file, &grammar2).unwrap();

    let mut file = File::create("jonla-parser/resources/bootstrap.bincode").unwrap();
    bincode::serialize_into(&mut file, &grammar2).unwrap();
}

// fn part1() {
//     let input = include_str!("../resources/meta.grammar");
//
//     let input_stream: StringStream = StringStream::new(input);
//     let result: Result<_, _> = run_parser_rule(&META_GRAMMAR, "toplevel", input_stream);
//     let result = match result {
//         Ok(o) => o.1,
//         Err(es) => {
//             for e in es {
//                 // print_tree_error(e, "file", input, true);
//                 print_set_error(e, "file", input, true);
//             }
//             return;
//         }
//     };
//
//     let mut file = File::create("jonla-parser-bootstrap/resources/temp.bincode").unwrap();
//     bincode::serialize_into(&mut file, &result).unwrap();
// }
//
// fn part2() {
//     let input = include_str!("../resources/meta.grammar");
//
//     // Leak because ownership was being annoying
//     let temp: &'static [u8] = Box::leak(
//         read("jonla-parser-bootstrap/resources/temp.bincode")
//             .unwrap()
//             .into_boxed_slice(),
//     );
//     let result: ActionResult<'static> = bincode::deserialize(&temp).unwrap();
//
//     let grammar2 = parse_grammarfile(&result, input);
//     let mut file = File::create("jonla-parser/resources/bootstrap.json").unwrap();
//     serde_json::to_writer_pretty(&mut file, &grammar2).unwrap();
// }
