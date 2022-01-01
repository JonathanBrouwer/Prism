#![feature(split_array)]
#![feature(box_syntax)]

pub mod peg;
mod lambday;
mod jonla;

use crate::jonla::jonla::{parse_jonla_program};
use crate::peg::parser::*;
use crate::peg::parser_success::*;

fn main() {
    let input = include_str!("../resources/example.jnl");
    let parsed = parse_jonla_program().parse((input, 0));
    let parsed = match parsed {
        Ok(ok) => ok,
        Err(err) => {
            err.0.print(input.to_string());
            return
        }
    };
    let name_checked = match parsed.result.transform_names() {
        Ok(ok) => ok,
        Err(err) => {
            err.print(input.to_string());
            return
        }
    };
    println!("{:?}", name_checked)



    // for term in ok.result {
    //     match term.type_check() {
    //         Ok(typ) => {
    //             println!("{:?}   :   {:?}", &term, typ);
    //         }
    //         Err(_) => {
    //             println!("{:?}   :   type error", &term);
    //         }
    //     }
    // }


}

