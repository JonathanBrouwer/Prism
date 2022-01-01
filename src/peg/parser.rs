use crate::jonla::jerror::JError;
use crate::ParseError;
use crate::peg::input::Input;
use crate::peg::parser_success::{ParseSuccess};

pub trait Parser<I: Input, O> {
    fn parse(&self, input: I) -> Result<ParseSuccess<I, O>, ParseError>;
}

impl<I: Input, O, F> Parser<I, O> for F
    where F: Fn(I) -> Result<ParseSuccess<I, O>, ParseError> {
    fn parse(&self, i: I) -> Result<ParseSuccess<I, O>, ParseError> {
        self(i)
    }
}

impl<I: Input, O> Parser<I, O> for Box<dyn Parser<I, O>> {
    fn parse(&self, i: I) -> Result<ParseSuccess<I, O>, ParseError> {
        (**self).parse(i)
    }
}