#![allow(dead_code)]

use prism_parser::error::error_printer::print_set_error;
use prism_parser::grammar::from_action_result::parse_grammarfile;
use prism_parser::grammar::grammar_ar::GrammarFile;
use prism_parser::parser::parser_instance::run_parser_rule;
use prism_parser::rule_action::action_result::ActionResult;
use prism_parser::rule_action::from_action_result::parse_rule_action;
use prism_parser::{parse_grammar, META_GRAMMAR};
use std::fs::{read, File};

fn get_new_grammar(input: &str) -> GrammarFile {
    match parse_grammar(input) {
        Ok(o) => o,
        Err(es) => {
            for e in es {
                // print_tree_error(e, "file", input, true);
                print_set_error(e, input, true);
            }
            panic!();
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

    let mut file = File::create("prism-parser/resources/bootstrap.json").unwrap();
    serde_json::to_writer_pretty(&mut file, &grammar2).unwrap();
    let mut file = File::create("prism-parser/resources/bootstrap.bincode").unwrap();
    bincode::serialize_into(&mut file, &grammar2).unwrap();
}

fn part1() {
    let input = include_str!("../resources/meta.grammar");

    let result: Result<_, _> = run_parser_rule(&META_GRAMMAR, "toplevel", input);
    let result = match result {
        Ok(o) => o,
        Err(es) => {
            for e in es {
                // print_tree_error(e, "file", input, true);
                print_set_error(e, input, true);
            }
            return;
        }
    };

    let mut file = File::create("prism-parser-bootstrap/resources/temp.bincode").unwrap();
    bincode::serialize_into(&mut file, &result).unwrap();
}

fn part2() {
    let input = include_str!("../resources/meta.grammar");

    // Leak because ownership was being annoying
    let temp: &'static [u8] = Box::leak(
        read("prism-parser-bootstrap/resources/temp.bincode")
            .unwrap()
            .into_boxed_slice(),
    );
    let result: ActionResult<'static> = bincode::deserialize(temp).unwrap();

    let grammar2: GrammarFile = parse_grammarfile(&result, input, parse_rule_action).unwrap();
    let mut file = File::create("prism-parser/resources/bootstrap.json").unwrap();
    serde_json::to_writer_pretty(&mut file, &grammar2).unwrap();
    let mut file = File::create("prism-parser/resources/bootstrap.bincode").unwrap();
    bincode::serialize_into(&mut file, &grammar2).unwrap();
}