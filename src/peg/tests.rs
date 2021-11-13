#[cfg(test)]
mod tests {
    use std::fmt::{Debug, Display, Formatter};
    use std::rc::Rc;
    use crate::peg::input::Input;
    use crate::peg::parser::PegParser;
    use crate::peg::parser_result::*;
    use crate::peg::parser_result::ParseErrorFlag::{NotAllInput, Recursive};
    use crate::peg::rules::{PegRule, TerminalPredicate};
    use crate::peg::tests::tests::TestInput::*;

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

    #[derive(Debug)]
    struct ExactPredicate {
        token: TestInput
    }

    impl TerminalPredicate<TestInput, TestInput> for ExactPredicate {
        fn run(&self, token: TestInput) -> bool {
            self.token == token
        }

        fn representitive(&self) -> TestInput {
            self.token
        }
    }

    struct Preds;
    impl Preds {
        pub fn exact(token: TestInput) -> Rc<dyn TerminalPredicate<TestInput, TestInput>> {
            Rc::new(ExactPredicate{ token })
        }
    }

    #[test]
    fn test_terminal1() {
        let rules = vec![
            PegRule::Terminal(Preds::exact(A)),
        ];
        assert_eq!(
            PegParser::new(rules.clone(), &[A]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("A")),
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[B]).parse_final(),
            Err(ParseError { positives: vec![A], flags: vec![], location: 0 }),
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[]).parse_final(),
            Err(ParseError { positives: vec![A], flags: vec![], location: 0 }),
        );
    }

    #[test]
    fn test_sequence1() {
        let rules = vec![
            PegRule::Terminal(Preds::exact(A)),
            PegRule::Sequence(vec![0, 0]),
        ];
        assert_eq!(
            PegParser::new(rules.clone(), &[A, A]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("[A A]"))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[B, A]).parse_final(),
            Err(ParseError { positives: vec![A], flags: vec![], location: 0 }),
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A, B]).parse_final(),
            Err(ParseError { positives: vec![A], flags: vec![], location: 1 }),
        );
    }

    #[test]
    fn test_choice1() {
        let rules = vec![
            PegRule::Terminal(Preds::exact(A)),
            PegRule::Terminal(Preds::exact(B)),
            PegRule::Choice(vec![0, 1]),
        ];
        assert_eq!(
            PegParser::new(rules.clone(), &[A]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("<0 A>")),
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[B]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("<1 B>")),
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[C]).parse_final(),
            Err(ParseError { positives: vec![B, A], flags: vec![], location: 0 }),
        );
    }

    #[test]
    fn test_rightrec() {
        let rules = vec![
            PegRule::Terminal(Preds::exact(A)),
            PegRule::Terminal(Preds::exact(B)),
            PegRule::Sequence(vec![0, 3]),
            PegRule::Choice(vec![2, 1]),
        ];
        assert_eq!(
            PegParser::new(rules.clone(), &[B]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("<1 B>"))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A, B]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("<0 [A <1 B>]>"))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A, A, B]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("<0 [A <0 [A <1 B>]>]>"))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[B, A]).parse_final(),
            Err(ParseError { positives: vec![], flags: vec![NotAllInput], location: 1 })
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A]).parse_final(),
            Err(ParseError { positives: vec![B, A], flags: vec![], location: 1 })
        );
    }

    #[test]
    fn test_leftrec() {
        let rules = vec![
            PegRule::Terminal(Preds::exact(A)),
            PegRule::Terminal(Preds::exact(B)),
            PegRule::Sequence(vec![3, 0]),
            PegRule::Choice(vec![2, 1]),
        ];
        assert_eq!(
            PegParser::new(rules.clone(), &[B]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("<1 B>"))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[B, A]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("<0 [<1 B> A]>"))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A, B]).parse_final(),
            Err(ParseError { positives: vec![B], flags: vec![Recursive], location: 0 })
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[A]).parse_final(),
            Err(ParseError { positives: vec![B], flags: vec![Recursive], location: 0 })
        );
    }

    #[test]
    fn test_leftrec_unavoidable() {
        let rules: Vec<PegRule<TestInput, TestInput>> = vec![
            PegRule::Sequence(vec![0])
        ];
        assert_eq!(
            PegParser::new(rules.clone(), &[A]).parse_final(),
            Err(ParseError { positives: vec![], flags: vec![Recursive], location: 0 })
        );
    }

    #[test]
    fn test_notall() {
        let rules: Vec<PegRule<TestInput, TestInput>> = vec![
            PegRule::Sequence(vec![])
        ];
        assert_eq!(
            PegParser::new(rules.clone(), &[A]).parse_final(),
            Err(ParseError { positives: vec![], flags: vec![NotAllInput], location: 0 })
        );
    }

    #[test]
    fn test_complex_terminal() {

        #[derive(Eq, PartialEq, Debug, Copy, Clone)]
        enum ErrorType {
            AOrB
        }
        #[derive(Eq, PartialEq, Debug, Copy, Clone)]
        struct ParseAOrB;
        impl Display for ErrorType {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "a or b")
            }
        }
        impl TerminalPredicate<TestInput, ErrorType> for ParseAOrB {
            fn run(&self, token: TestInput) -> bool {
                token == A || token == B
            }

            fn representitive(&self) -> ErrorType {
                ErrorType::AOrB
            }
        }
        let rules = vec![
            PegRule::Terminal(Rc::new(ParseAOrB{})),
        ];
        assert_eq!(
            PegParser::new(rules.clone(), &[A]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("A"))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[B]).parse_final().map(|ok| ok.to_string()),
            Ok(String::from("B"))
        );
        assert_eq!(
            PegParser::new(rules.clone(), &[C]).parse_final().map(|ok| ok.to_string()),
            Err(ParseError { positives: vec![ErrorType::AOrB], flags: vec![], location: 0 })
        );

    }
}