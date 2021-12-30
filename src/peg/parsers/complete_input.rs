use crate::{Parser};
use crate::peg::input::Input;

pub fn complete_input<I: Input, O, P: Parser<I, O>>(
    parser: P,
) -> impl Parser<I, O> {
    move |pos: I| {
        let ok = parser.parse(pos)?;
        if ok.pos.next().is_err() {
            Ok(ok)
        } else {
            Err(ok.best_error.unwrap())
        }
    }
}