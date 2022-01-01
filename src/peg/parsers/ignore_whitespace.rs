use crate::{Parser};
use crate::peg::input::{Input};

pub fn ignore_whitespace<'a, O, P: 'a + Parser<'a, O>>(
    parser: P,
) -> impl Parser<'a, O> {
    move |mut pos: Input<'a>| {
        while let Ok(ok) = pos.next() {
            if ok.result.is_whitespace() {
                pos.pos = ok.pos;
            } else {
                break;
            }
        }
        parser.parse(pos)
    }
}

