use crate::{Parser};
use crate::peg::input::{Input};

pub fn complete_input<'a, O, P: 'a + Parser<'a, O>>(
    parser: P,
) -> impl Parser<'a, O> {
    move |pos: Input<'a>| {
        let ok = parser.parse(pos)?;
        if pos.src.len() == ok.pos {
            Ok(ok)
        } else {
            Err(ok.best_error.unwrap())
        }
    }
}