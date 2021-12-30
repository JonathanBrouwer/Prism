use miette::Severity;
use crate::{ParseError, ParseErrorEntry, ParseErrorLabel, Parser, ParseSuccess};
use crate::peg::input::Input;

pub fn exact<I: Input>(
    exact: Vec<I::InputElement>
) -> impl Parser<I, Vec<I::InputElement>> {
    move |mut pos: I| {
        let mut result = vec![];
        let mut best_error = None;

        for elem in &exact {
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

