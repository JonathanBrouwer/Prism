use crate::{ParseError, Parser, ParseSuccess};
use crate::peg::input::Input;

pub fn alt<I: Input, O, P: Parser<I, O>>(
    parsers: Vec<P>
) -> impl Parser<I, O> {
    move |pos: I| {
        let mut best_error = None;
        for parser in &parsers {
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

