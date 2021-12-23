use miette::Severity;
use crate::{ParseError, ParseErrorEntry, ParseErrorLabel, Parser, ParseSuccess};
use crate::peg::input::Input;

pub struct Choice<I: Input, O> {
    parsers: Vec<Box<dyn Parser<I, O>>>
}

impl<I: Input, O> Parser<I, O> for Choice<I, O> {
    fn parse(&self, pos: I) -> Result<ParseSuccess<I, O>, ParseError<I>> {
        let mut best_error = None;
        for parser in &self.parsers {
            match parser.parse(pos.clone()) {
                Ok(suc) => {
                    best_error = ParseError::parse_error_combine_opt2(best_error, suc.best_error);
                    return Ok(ParseSuccess { result: suc.result, pos: suc.pos, best_error })
                }
                Err(err) => {
                    best_error = Some(ParseError::parse_error_combine_opt1(err, best_error))
                }
            }
        }
        return Err(best_error.unwrap());
    }
}

