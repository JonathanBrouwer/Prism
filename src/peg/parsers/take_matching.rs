use crate::{Parser, ParseSuccess};
use crate::jonla::jerror::{JError, JErrorEntry};
use crate::peg::input::{Input};

pub fn take_matching<'a>(
    name: String,
    matching_fun: Box<dyn Fn(char) -> bool>
) -> impl Parser<'a, char> {
    move |pos: Input<'a>| {
        if let Ok(ps@ParseSuccess{ result, .. }) = pos.next() {
            if matching_fun(result) {
                return Ok(ps)
            }
        }
        return Err((JError{ errors: vec![JErrorEntry::UnexpectedString((pos.pos, pos.pos + 1), name.clone())]}, pos.pos))
    }
}

