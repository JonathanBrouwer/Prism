#![feature(split_array)]

pub mod peg;
mod lambday;
mod jonla;

use crate::jonla::jonla::{JonlaTerm, parse_jonla_program};
use crate::peg::parser::*;
use crate::peg::parser_result::*;

fn main() {
    let input = include_str!("../resources/example.jnl");
    let parsed = parse_jonla_program().parse((input, 0));
    match parsed {
        Ok(ok) => {
            for term in ok.result {
                println!("{:?}", term)
            }
        }
        Err(err) => {
            println!("{}", err);
            return
        }
    }


}

