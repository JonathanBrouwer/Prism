use std::fmt::{Debug, Display};
use miette::Severity;
use crate::{ParseError, ParseErrorEntry, ParseErrorLabel, ParseSuccess};

pub trait Input: Sized + Clone {
    type InputElement: Debug + Display + PartialEq + Eq + Clone + Copy;

    fn next(&self) -> Result<ParseSuccess<Self, Self::InputElement>, ParseError<Self>>;
    fn pos(&self) -> usize;

    fn src_str<'a>(&'a self) -> Box<dyn ToString + 'a>;
    fn src_slice(&self) -> (usize, usize);
}

impl Input for (&str, usize) {
    type InputElement = char;

    fn next(&self) -> Result<ParseSuccess<Self, Self::InputElement>, ParseError<Self>> {
        if self.1 < self.0.len() {
            let c = self.0[self.1..].chars().next().unwrap();
            Ok(ParseSuccess {
                result: c,
                best_error: None,
                pos: (self.0, self.1 + c.len_utf8())
            })
        } else {
            let label = ParseErrorLabel { msg: "Unexpected end of file".to_string(), at: self.pos() };
            let entry = ParseErrorEntry { msg: "Parsing error".to_string(), severity: Severity::Error, labels: vec![label] };
            Err(ParseError {
                errors: vec![entry],
                pos: *self
            })
        }
    }

    fn pos(&self) -> usize {
        self.1
    }

    fn src_str<'a>(&'a self) -> Box<dyn ToString + 'a> {
        Box::new(self.0)
    }

    fn src_slice(&self) -> (usize, usize) {
        (self.1, 1)
    }
}
