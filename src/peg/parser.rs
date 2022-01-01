use crate::ParseError;
use crate::peg::input::{InputNew};
use crate::peg::parser_success::{ParseSuccess};

pub trait Parser<'a, O> {
    fn parse(&self, input: InputNew<'a>) -> Result<ParseSuccess<O>, ParseError>;
}

impl<'a, O, F> Parser<'a, O> for F
    where
        F: Fn(InputNew<'a>) -> Result<ParseSuccess<O>, ParseError> + 'a,
{
    fn parse(&self, input: InputNew<'a>) -> Result<ParseSuccess<O>, ParseError> { self(input) }
}

impl<'a, O> Parser<'a, O> for Box<dyn Parser<'a, O>> {
    fn parse(&self, i: InputNew<'a>) -> Result<ParseSuccess<O>, ParseError> {
        (**self).parse(i)
    }
}