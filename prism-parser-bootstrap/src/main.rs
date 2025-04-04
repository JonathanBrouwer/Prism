#![allow(dead_code)]

use bumpalo::Bump;
use prism_parser::core::allocs::Allocs;
use prism_parser::error::aggregate_error::ParseResultExt;
use prism_parser::error::set_error::SetError;
use prism_parser::parse_grammar;
use std::fs::File;

fn main() {
    normal();
    // part1();
    // part2();
}

fn normal() {
    let input = include_str!("../resources/meta.pg");
    let bump = Bump::new();
    let alloc = Allocs::new(&bump);
    let grammar2 = parse_grammar::<SetError>(input, alloc).unwrap_or_eprint();

    // let grammar: &'static GrammarFile = &META_GRAMMAR;
    // assert_eq!(grammar, &grammar2); // Safety check

    let mut file = File::create("prism-parser/resources/bootstrap.json").unwrap();
    serde_json::to_writer_pretty(&mut file, &grammar2).unwrap();
    let mut file = File::create("prism-parser/resources/bootstrap.msgpack").unwrap();
    rmp_serde::encode::write_named(&mut file, &grammar2).unwrap();
}

// fn part1() {
//     let input = include_str!("../resources/meta.pg");
//
//     run_parser_rule_here!(result = &META_GRAMMAR, "toplevel", SetError, input);
//     let result = result.unwrap_or_eprint().into_value::<ActionResult<'arn>>();
//
//     let mut file = File::create("prism-parser-bootstrap/resources/temp.bincode").unwrap();
//     bincode::serialize_into(&mut file, &result).unwrap();
// }
//
// fn part2() {
//     let input = include_str!("../resources/meta.pg");
//
//     // Leak because ownership was being annoying
//     let temp: &'static [u8] = Box::leak(
//         read("prism-parser-bootstrap/resources/temp.bincode")
//             .unwrap()
//             .into_boxed_slice(),
//     );
//     let result: ActionResult<'_, 'static> = bincode::deserialize(temp).unwrap();
//
//     let bump = Bump::new();
//     let alloc = Allocs::new(&bump);
//     let grammar2: GrammarFile = parse_grammarfile(&result, input, alloc, |ar, src| {
//         parse_rule_action(ar, src, alloc)
//     })
//     .unwrap();
//     let mut file = File::create("prism-parser/resources/bootstrap.json").unwrap();
//     serde_json::to_writer_pretty(&mut file, &grammar2).unwrap();
//     let mut file = File::create("prism-parser/resources/bootstrap.bincode").unwrap();
//     bincode::serialize_into(&mut file, &grammar2).unwrap();
// }
