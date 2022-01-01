use crate::{parse_error_combine_opt1, parse_error_combine_opt2, Parser, ParseSuccess};
use crate::peg::input::Input;
use crate::peg::parsers::take_matching::take_matching;

pub fn repeat_m_n<'a, O, P: 'a + Parser<'a, O>>(
    parser: P,
    min_count: usize,
    max_count: usize
) -> impl Parser<'a, Vec<O>> {
    move |mut pos: Input<'a>| {
        let mut result = vec![];
        let mut best_error = None;

        for _ in 0..min_count {
            let res = parser.parse(pos)?;
            result.push(res.result);
            pos.pos = res.pos;
            best_error = parse_error_combine_opt2(best_error, res.best_error);
        }

        for _ in min_count..max_count {
            match parser.parse(pos.clone()) {
                Ok(ok) => {
                    result.push(ok.result);
                    pos.pos = ok.pos;
                    best_error = parse_error_combine_opt2(best_error, ok.best_error);
                }
                Err(err) => {
                    best_error = Some(parse_error_combine_opt1(err, best_error));
                    break;
                }
            }
        }

        Ok(ParseSuccess { result, best_error, pos: pos.pos })
    }
}

pub fn repeat_m_n_matching<'a>(
    name: String,
    matching_fun: Box<dyn Fn(char) -> bool>,
    min_count: usize,
    max_count: usize
) -> impl Parser<'a, Vec<char>> {
    repeat_m_n(take_matching(name, matching_fun), min_count, max_count)
}
