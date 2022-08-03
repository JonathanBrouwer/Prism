use jonla_macros::parser::core::presult::PResult::*;
use crate::autogen::parse::parse_term;

#[allow(unused)]
#[rustfmt::skip]
mod autogen;

fn main() {
    let input = include_str!("../resources/program.jnl");
    let result = parse_term(input);
    match result {
        POk(o, _) => {
            println!("{:?}", o);
        }
        PRec(errs, o, _) => {
            for err in errs {
                println!("{:?}", err);
            }

            println!("Recovered result: ");
            println!("{:?}", o);
        }
        PErr(errs, err, _) => {
            for err in errs {
                println!("{:?}", err);
            }
            println!("{:?}", err);
        }
    }
}