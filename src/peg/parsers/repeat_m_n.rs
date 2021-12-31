use crate::{Parser, ParseSuccess};
use crate::jonla::jerror::JError;
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
            best_error = JError::parse_error_combine_opt2(best_error, res.best_error);
        }

        for _ in min_count..max_count {
            match parser.parse(pos.clone()) {
                Ok(ok) => {
                    result.push(ok.result);
                    pos = ok.pos;
                    best_error = JError::parse_error_combine_opt2(best_error, ok.best_error);
                }
                Err(err) => {
                    best_error = Some(JError::parse_error_combine_opt1(err, best_error));
                    break;
                }
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
    use crate::jonla::jerror::JError;
    use crate::jonla::jerror::JErrorEntry::UnexpectedString;
    use crate::ParseSuccess;
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
            ParseSuccess { result: vec!['a', 'b', 'b', 'a', 'b'], best_error: Some(JError { errors: vec![UnexpectedString((inp, 5), "a or b".to_string())], pos: ("abbabx", 5) }), pos: (inp, 5) }
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
            JError { errors: vec![UnexpectedString((inp, 5), "a or b".to_string())], pos: (inp, 5) }
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