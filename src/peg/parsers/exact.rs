use miette::Severity;
use crate::{ParseError, ParseErrorEntry, ParseErrorLabel, Parser, ParseSuccess};
use crate::peg::input::Input;

pub fn exact<I: Input>(
    elems: Vec<I::InputElement>
) -> impl Parser<I, Vec<I::InputElement>> {
    move |mut pos: I| {
        let mut result = vec![];
        let mut best_error = None;

        for elem in &elems {
            let res = pos.next()?;
            if res.result != *elem {
                let label = ParseErrorLabel { msg: format!("Expected {} here", elem), at: pos.pos() };
                let entry = ParseErrorEntry { msg: "Parsing error".to_string(), severity: Severity::Error, labels: vec![label] };
                return Err(ParseError{ errors: vec![entry], pos })
            }
            result.push(res.result);
            pos = res.pos;
            best_error = ParseError::parse_error_combine_opt2(best_error, res.best_error);
        }

        Ok(ParseSuccess { result, best_error, pos })
    }
}

pub fn exact_str<I: Input<InputElement=char>>(
    str: &'static str
) -> impl Parser<I, String> {
    move |pos: I| {
        match exact(str.chars().collect()).parse(pos).map(|ok| ok.map(|r| r.into_iter().collect())) {
            Ok(ok) => Ok(ok),
            Err(err) => {
                let label = ParseErrorLabel { msg: format!("Expected {} here", str), at: err.pos.pos() };
                let entry = ParseErrorEntry { msg: "Parsing error".to_string(), severity: Severity::Error, labels: vec![label] };
                return Err(ParseError{ errors: vec![entry], pos: err.pos })
            }
        }
    }
}