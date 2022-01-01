use std::fmt::{Debug};
use crate::jonla::jerror::{JError, JErrorEntry};
use crate::{ParseError, ParseSuccess};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct InputNew<'a> {
    pub src: &'a str,
    pub pos: usize
}

impl<'a> InputNew<'a> {
    pub fn next(&self) -> Result<ParseSuccess<char>, ParseError> {
        if self.pos < self.src.len() {
            let c = self.src[self.pos..].chars().next().unwrap();
            Ok(ParseSuccess {
                result: c,
                best_error: None,
                pos: self.pos + c.len_utf8()
            })
        } else {
            Err((JError { errors: vec![JErrorEntry::UnexpectedEOF((self.clone().pos, self.clone().pos + 1))]}, self.pos))
        }
    }
}