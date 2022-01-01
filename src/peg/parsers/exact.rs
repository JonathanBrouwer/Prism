use crate::{Parser, ParseSuccess};
use crate::jonla::jerror::{JError, JErrorEntry};
use crate::peg::input::Input;

pub fn exact_str<'a>(
    str: &'static str
) -> impl Parser<'a, ()> {
    move |startpos: Input<'a>| {
        if startpos.src[startpos.pos..].starts_with(str) {
            Ok(ParseSuccess {
                result: (),
                best_error: None,
                pos: startpos.pos + str.len()
            })
        } else {
            Err((JError { errors: vec![JErrorEntry::UnexpectedStr((startpos.pos, startpos.pos + str.len()), str)]}, startpos.pos))
        }
    }
}