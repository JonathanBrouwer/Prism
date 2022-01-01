use crate::{parse_error_combine_opt1, parse_error_combine_opt2, Parser, ParseSuccess};
use crate::peg::input::{Input};

pub fn alt<'a, O, P: 'a + Parser<'a, O>>(
    parsers: Vec<P>
) -> impl Parser<'a, O> {
    move |pos: Input<'a>| {
        let mut best_error = None;
        for parser in &parsers {
            match parser.parse(pos.clone()) {
                Ok(suc) => {
                    best_error = parse_error_combine_opt2(best_error, suc.best_error);
                    return Ok(ParseSuccess { result: suc.result, pos: suc.pos, best_error })
                }
                Err(err) => {
                    best_error = Some(parse_error_combine_opt1(err, best_error))
                }
            }
        }
        return Err(best_error.unwrap());
    }
}

