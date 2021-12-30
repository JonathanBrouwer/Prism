use crate::{Parser};
use crate::peg::input::Input;

pub fn ignore_whitespace<I: Input<InputElement=char>, O, P: Parser<I, O>>(
    parser: P,
) -> impl Parser<I, O> {
    move |mut pos: I| {
        while let Ok(ok) = pos.next() {
            if ok.result.is_whitespace() {
                pos = ok.pos;
            } else {
                break;
            }
        }
        parser.parse(pos)
    }
}

