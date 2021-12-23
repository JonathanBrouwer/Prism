use miette::Severity;
use crate::peg::input::Input;
use crate::peg::parser_result::{ParseError, ParseErrorEntry, ParseErrorLabel, ParseSuccess};

pub trait Parser<I: Input, O> {
    fn parse(&self, input: I) -> Result<ParseSuccess<I, O>, ParseError<I>>;
}
