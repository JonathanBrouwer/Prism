#![feature(box_syntax)]

use crate::grammar::GrammarFile;
use std::path::PathBuf;

mod codegen;
mod formatting_file;
pub mod grammar;
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
