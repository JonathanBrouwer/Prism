use miette::Severity;
use crate::{ParseError, ParseErrorEntry, ParseErrorLabel, Parser, ParseSuccess};
use crate::peg::input::Input;

pub struct IgnoreWhitespaceAround<I: Input<InputElement=char>, O> {
    sub_parser: Box<dyn Parser<I, O>>
}

impl<I: Input<InputElement=char>, O> Parser<I, O> for IgnoreWhitespaceAround<I, O> {
    fn parse(&self, mut input: I) -> Result<ParseSuccess<I, O>, ParseError<I>> {
        while let Ok(suc) = input.next() {
            if suc.result.is_whitespace() { input = suc.pos; continue }
            return self.sub_parser.parse(input)
        }
        return Err(input.next().err().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use crate::Parser;
    use crate::peg::parsers::ignore_whitespace_around::IgnoreWhitespaceAround;
    use crate::peg::parsers::matching_element::*;

    #[test]
    fn test_whitespace_around() {
        let mut inp = ("ab cd ef", 0);
        let parser = IgnoreWhitespaceAround { sub_parser: Box::new(MatchingElement { matching_fun: Box::new(|c| true), name: "anything".to_string() })};
        let mut results = vec![];
        while let Ok(suc) = parser.parse(inp) {
            results.push(suc.result);
            inp = suc.pos;
        }
        assert_eq!(vec!['a', 'b', 'c', 'd', 'e', 'f'], results);
    }
}