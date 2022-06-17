#![feature(box_syntax)]

use std::path::PathBuf;
use crate::grammar::GrammarFile;

pub mod grammar;
mod codegen;
mod formatting_file;
mod parser;

pub fn handle_language(path: PathBuf) {
    let s = std::fs::read_to_string(path.clone()).unwrap();
    let grammar: GrammarFile = match grammar::grammar_def::toplevel(&s) {
        Ok(ok) => ok,
        Err(err) => {
            panic!("{}", err);
        }
    };

    codegen::codegen(&grammar);

    println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
    println!("cargo:rerun-if-changed=src/autogen/");
}

