use miette::Severity;
use crate::{ParseError, ParseErrorEntry, ParseErrorLabel, Parser, ParseSuccess};
use crate::peg::input::Input;

pub struct Chain<I: Input, O> {
    parsers: Vec<Box<dyn Parser<I, O>>>
}

impl<I: Input, O> Parser<I, Vec<O>> for Chain<I, O> {
    fn parse(&self, mut pos: I) -> Result<ParseSuccess<I, Vec<O>>, ParseError<I>> {
        let mut result = vec![];
        let mut best_error = None;
        for parser in &self.parsers {
            let res = parser.parse(pos)?;
            result.push(res.result);
            pos = res.pos;
            best_error = ParseError::parse_error_combine_opt2(best_error, res.best_error);
        }
        Ok(ParseSuccess { result, best_error, pos })
    }
}

