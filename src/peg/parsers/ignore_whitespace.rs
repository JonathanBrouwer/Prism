use crate::{Parser};
use crate::peg::input::{InputNew};

pub fn ignore_whitespace<'a, O, P: 'a + Parser<'a, O>>(
    parser: P,
) -> impl Parser<'a, O> {
    move |mut pos: InputNew<'a>| {
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

