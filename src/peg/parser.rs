use miette::Severity;
use crate::peg::input::Input;
use crate::peg::parser_result::{ParseError, ParseErrorEntry, ParseErrorLabel, ParseSuccess};

pub trait Parser<I: Input, O> {
    fn parse(&self, input: I) -> Result<ParseSuccess<I, O>, ParseError<I>>;
}

pub struct AnyElement;

impl<I: Input> Parser<I, I::InputElement> for AnyElement {
    fn parse(&self, input: I) -> Result<ParseSuccess<I, I::InputElement>, ParseError<I>> {
        match input.next() {
            None => {
                let label = ParseErrorLabel { msg: "Expected any character here.".to_string(), at: input.pos() };
                let entry = ParseErrorEntry { msg: "Parsing error".to_string(), severity: Severity::Error, labels: vec![label] };
                Err(ParseError{ errors: vec![entry], pos: input })
            },
            Some((result, rest)) => Ok(ParseSuccess{
                result,
                best_error: None,
                pos: rest
            })
        }
    }
}

pub struct OneElement<I: Input> {
    pub(crate) element: I::InputElement
}

impl<I: Input> Parser<I, I::InputElement> for OneElement<I> {
    fn parse(&self, input: I) -> Result<ParseSuccess<I, I::InputElement>, ParseError<I>> {
        if let Some((result, rest)) = input.next() {
            if self.element == result {
                return Ok(ParseSuccess{
                    result,
                    best_error: None,
                    pos: rest
                })
            }
        }
        let label = ParseErrorLabel { msg: format!("Expected {} here", self.element), at: input.pos() };
        let entry = ParseErrorEntry { msg: "Parsing error".to_string(), severity: Severity::Error, labels: vec![label] };
        return Err(ParseError{ errors: vec![entry], pos: input })
    }
}

// pub struct MatchingElement<I: Input> {
//     matching_fun: Box<dyn FnMut(I::InputElement) -> bool>
// }
//
// impl



#[cfg(test)]
mod tests {
    use std::fmt::{Display, Formatter};
    use crate::peg::parser::*;
    use crate::peg::parser::tests::TestInput::*;

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
    fn test_any_element1() {
        let inp = "ab";
        assert_eq!(
            AnyElement{}.parse((inp, 0)).unwrap(),
            ParseSuccess{result: 'a', best_error: None, pos: (inp, 1) }
        );
    }

    #[test]
    fn test_any_element2() {
        let inp = "a";
        assert_eq!(
            AnyElement{}.parse((inp, 0)).unwrap(),
            ParseSuccess{result: 'a', best_error: None, pos: (inp, 1) }
        );
    }

    #[test]
    fn test_any_element3() {
        let inp = "";
        assert_eq!(
            AnyElement{}.parse((inp, 0)).unwrap_err(),
            ParseError { errors: vec![ParseErrorEntry { msg: "Parsing error".to_string(), severity: Severity::Error, labels: vec![ParseErrorLabel { msg: "Expected any character here.".to_string(), at: 0 }] }], pos: (inp, 0) }
        );
    }

    // #[test]
    // fn test_specific_element() {
    //     assert_eq!(
    //         OneElement{element: A}.parse([A, B].as_slice()).unwrap(),
    //         ParseSuccess{result: A, best_error: None, rest: [B].as_slice() }
    //     );
    //     assert_eq!(
    //         OneElement{element: B}.parse([A, B].as_slice()).unwrap_err(),
    //         ParseError {positives: vec![B], flags: vec![], len_left: 2 }
    //     );
    //     assert_eq!(
    //         OneElement{element: A}.parse([A].as_slice()).unwrap(),
    //         ParseSuccess{result: A, best_error: None, rest: [].as_slice() }
    //     );
    //     assert_eq!(
    //         OneElement{element: B}.parse([A].as_slice()).unwrap_err(),
    //         ParseError {positives: vec![B], flags: vec![], len_left: 1 }
    //     );
    //     assert_eq!(
    //         OneElement{element: A}.parse([].as_slice()).unwrap_err(),
    //         ParseError {positives: vec![A], flags: vec![], len_left: 0 }
    //     );
    // }
}