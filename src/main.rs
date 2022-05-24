#![feature(box_syntax)]
#![allow(clippy::needless_lifetimes)]

pub mod beta_eq;
pub mod env;
pub mod parser;
pub mod term;
pub mod type_check;
pub mod core_parser;

// use crate::env::{Env, RcEnv};
// use crate::term::Term;
// use crate::type_check::type_check;
// use parser::term_parser;

fn main() {
    // let term: Term = term_parser::program(include_str!("../resources/program.dl")).unwrap();
    // println!("Program:\n{:?}", term);
    //
    // let typ = type_check(&term, &Env::empty()).unwrap();
    // println!("Type: {:?}", typ.0);
    // println!("Type env: {}", typ.1.debug());
}
