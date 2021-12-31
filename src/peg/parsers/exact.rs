use crate::{Parser, ParseSuccess};
use crate::jonla::jerror::{JError, JErrorEntry};
use crate::peg::input::Input;

pub fn exact<I: Input>(
    elems: Vec<I::InputElement>
) -> impl Parser<I, Vec<I::InputElement>> {
    move |startpos: I| {
        let mut pos = startpos;
        let mut result = vec![];
        let mut best_error = None;

        for elem in &elems {
            let res = pos.next()?;
            if res.result != *elem {
                return Err(JError { errors: vec![JErrorEntry::UnexpectedString((startpos.pos(), startpos.pos() + elems.len()), elem.to_string())], pos })
            }
            result.push(res.result);
            pos = res.pos;
            best_error = JError::parse_error_combine_opt2(best_error, res.best_error);
        }

        Ok(ParseSuccess { result, best_error, pos })
    }
}

pub fn exact_str<I: Input<InputElement=char>>(
    str: &'static str
) -> impl Parser<I, String> {
    move |startpos: I| {
        match exact(str.chars().collect()).parse(startpos).map(|ok| ok.map(|r| r.into_iter().collect())) {
            Ok(ok) => Ok(ok),
            Err(err) => {
                return Err(JError { errors: vec![JErrorEntry::UnexpectedStr((startpos.pos(), startpos.pos() + str.len()), str)], pos: err.pos })
            }
        }
    }
}