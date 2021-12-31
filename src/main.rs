#![feature(split_array)]

pub mod peg;
mod lambday;
mod jonla;

use crate::jonla::jonla::{parse_jonla_program};
use crate::peg::parser::*;
use crate::peg::parser_success::*;

fn main() {
    let input = include_str!("../resources/example.jnl");
    let parsed = parse_jonla_program().parse((input, 0));
    let ok = match parsed {
        Ok(ok) => { ok }
        Err(err) => {
            println!("{}", err);
            return
        }
    };

    for term in ok.result {
        match term.type_check() {
            Ok(typ) => {
                print!("{:?}   :   {:?}", &term, typ);
            }
            Err(_) => {
                print!("{:?}   :   type error", &term);
            }
        }
    }


}

