use crate::{ParseError, Parser, ParseSuccess};
use crate::peg::input::Input;
use crate::peg::parsers::take_matching::take_matching;

pub fn repeat_m_n<I: Input, O, P: Parser<I, O>>(
    parser: P,
    min_count: usize,
    max_count: usize
) -> impl Parser<I, Vec<O>> {
    move |mut pos: I| {
        let mut result = vec![];
        let mut best_error = None;

        for _ in 0..min_count {
            let res = parser.parse(pos)?;
            result.push(res.result);
            pos = res.pos;
            best_error = ParseError::parse_error_combine_opt2(best_error, res.best_error);
        }

        for _ in min_count..max_count {
            if let Ok(res) = parser.parse(pos.clone()) {
                result.push(res.result);
                pos = res.pos;
                best_error = ParseError::parse_error_combine_opt2(best_error, res.best_error);
            } else {
                break
            }
        }

        Ok(ParseSuccess { result, best_error, pos })
    }
}

pub fn repeat_m_n_matching<I: Input>(
    name: String,
    matching_fun: Box<dyn Fn(I::InputElement) -> bool>,
    min_count: usize,
    max_count: usize
) -> impl Parser<I, Vec<I::InputElement>> {
    repeat_m_n(take_matching(name, matching_fun), min_count, max_count)
}

#[cfg(test)]
mod tests {
    use miette::Severity;
    use crate::{ParseError, ParseErrorEntry, ParseErrorLabel, ParseSuccess};
    use crate::peg::parsers::repeat_m_n::repeat_m_n_matching;
    use crate::peg::parser::Parser;

    #[test]
    fn test_matching_element1() {
        let inp = "abbabx";
        assert_eq!(
            repeat_m_n_matching(
                "a or b".to_string(),
                Box::new(|c| c == 'a' || c == 'b'),
                0,
                usize::MAX
            ).parse((inp, 0)).unwrap(),
            ParseSuccess { result: vec!['a', 'b', 'b', 'a', 'b'], best_error: None, pos: (inp, 5) }
        );
    }

    #[test]
    fn test_matching_element2() {
        let inp = "abbabx";
        assert_eq!(
            repeat_m_n_matching(
                "a or b".to_string(),
                Box::new(|c| c == 'a' || c == 'b'),
                6,
                usize::MAX
            ).parse((inp, 0)).unwrap_err(),
            ParseError { errors: vec![ParseErrorEntry { msg: "Parsing error".to_string(), severity: Severity::Error, labels: vec![ParseErrorLabel { msg: "Expected a or b here".to_string(), at: 5 }] }], pos: (inp, 5) }
        );
    }

    #[test]
    fn test_matching_element3() {
        let inp = "abbabx";
        assert_eq!(
            repeat_m_n_matching(
                "a or b".to_string(),
                Box::new(|c| c == 'a' || c == 'b'),
                0,
                3
            ).parse((inp, 0)).unwrap(),
            ParseSuccess { result: vec!['a', 'b', 'b'], best_error: None, pos: (inp, 3) }
        );
    }

}