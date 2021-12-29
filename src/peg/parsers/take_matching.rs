use miette::Severity;
use crate::{ParseError, ParseErrorEntry, ParseErrorLabel, Parser, ParseSuccess};
use crate::peg::input::Input;

pub fn take_matching<I: Input>(
    name: String,
    matching_fun: Box<dyn Fn(I::InputElement) -> bool>
) -> impl Parser<I, I::InputElement> {
    move |pos: I| {
        if let Ok(ps@ParseSuccess{ result, .. }) = pos.next() {
            if matching_fun(result) {
                return Ok(ps)
            }
        }
        let label = ParseErrorLabel { msg: format!("Expected {} here", name), at: pos.pos() };
        let entry = ParseErrorEntry { msg: "Parsing error".to_string(), severity: Severity::Error, labels: vec![label] };
        return Err(ParseError{ errors: vec![entry], pos })
    }
}

