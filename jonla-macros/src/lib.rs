#![feature(box_syntax)]

use std::path::PathBuf;
use crate::grammar::GrammarFile;

mod grammar;
mod codegen;
mod formatting_file;

pub fn handle_language(path: PathBuf) {
    let s = std::fs::read_to_string(path.clone()).unwrap();
    let grammar: GrammarFile = grammar::grammar_def::toplevel(&s).unwrap();

    codegen::codegen(&grammar);

    println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
    println!("cargo:rerun-if-changed=src/autogen/");
}

