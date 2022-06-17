#[allow(unused)]
#[rustfmt::skip]
mod autogen;

use std::path::{Path, PathBuf};
use jonla_macros::{grammar, handle_language};
use jonla_macros::grammar::GrammarFile;

fn main() {
    let s = include_str!("../resources/grammar.jonla_peg");
    let grammar: GrammarFile = match grammar::grammar_def::toplevel(&s) {
        Ok(ok) => ok,
        Err(err) => {
            panic!("{}", err);
        }
    };
    println!("");
}
