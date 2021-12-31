use crate::jonla::jerror::JError;
use crate::peg::input::Input;
use crate::peg::parser_success::{ParseSuccess};

pub trait Parser<I: Input, O> {
    fn parse(&self, input: I) -> Result<ParseSuccess<I, O>, JError<I>>;
}

impl<I: Input, O, F> Parser<I, O> for F
    where F: Fn(I) -> Result<ParseSuccess<I, O>, JError<I>> {
    fn parse(&self, i: I) -> Result<ParseSuccess<I, O>, JError<I>> {
        self(i)
    }
}

impl<I: Input, O> Parser<I, O> for Box<dyn Parser<I, O>> {
    fn parse(&self, i: I) -> Result<ParseSuccess<I, O>, JError<I>> {
        (**self).parse(i)
    }
}