#![feature(split_array)]

pub mod peg;

use miette::Severity;
use crate::peg::parser::*;
use crate::peg::parser_result::*;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

fn main() {
    let input = "abc\ndef";
    let err: ParseError<_> = OneElement{ element: 'x' }.parse((input, 0)).unwrap_err();
    println!("{}", err);



}

