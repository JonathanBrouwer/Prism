use miette::Severity;
use crate::{ParseError, ParseErrorEntry, ParseErrorLabel, Parser, ParseSuccess};
use crate::peg::input::Input;

pub struct MatchingElement<I: Input> {
    pub(crate) matching_fun: Box<dyn Fn(I::InputElement) -> bool>,
    pub(crate) name: String,
}

impl<I: Input> Parser<I, I::InputElement> for MatchingElement<I> {
    fn parse(&self, input: I) -> Result<ParseSuccess<I, I::InputElement>, ParseError<I>> {
        if let Ok(ps@ParseSuccess{ result, .. }) = input.next() {
            if (*self.matching_fun)(result) {
                return Ok(ps)
            }
        }
        let label = ParseErrorLabel { msg: format!("Expected {} here", self.name), at: input.pos() };
        let entry = ParseErrorEntry { msg: "Parsing error".to_string(), severity: Severity::Error, labels: vec![label] };
        return Err(ParseError{ errors: vec![entry], pos: input })
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::{Display, Formatter};
    use miette::Severity;
    use crate::peg::parsers::matching_element::*;

    #[derive(Eq, PartialEq, Debug, Copy, Clone)]
    enum TestInput {
        A,
        B,
        C,
    }

    impl Display for TestInput {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                TestInput::A => write!(f, "A"),
                TestInput::B => write!(f, "B"),
                TestInput::C => write!(f, "C"),
            }
        }
    }

    #[test]
    fn test_matching_element1() {
        let inp = "ax";
        assert_eq!(
            MatchingElement{matching_fun: Box::new(|c| c == 'a' || c == 'b'), name: "a or b".to_string()}.parse((inp, 0)).unwrap(),
            ParseSuccess{result: 'a', best_error: None, pos: (inp, 1) }
        );
    }

    #[test]
    fn test_matching_element2() {
        let inp = "bx";
        assert_eq!(
            MatchingElement{matching_fun: Box::new(|c| c == 'a' || c == 'b'), name: "a or b".to_string()}.parse((inp, 0)).unwrap(),
            ParseSuccess{result: 'b', best_error: None, pos: (inp, 1) }
        );
    }

    #[test]
    fn test_matching_element3() {
        let inp = "cx";
        assert_eq!(
            MatchingElement{matching_fun: Box::new(|c| c == 'a' || c == 'b'), name: "a or b".to_string()}.parse((inp, 0)).unwrap_err(),
            ParseError { errors: vec![ParseErrorEntry { msg: "Parsing error".to_string(), severity: Severity::Error, labels: vec![ParseErrorLabel { msg: "Expected a or b here".to_string(), at: 0 }] }], pos: (inp, 0) }
        );
    }
}