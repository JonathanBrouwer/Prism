#[cfg(test)]
mod tests {
    use crate::peg_parser::peg_parser::*;
    use std::collections::HashMap;
    use crate::peg_parser::nice_rules::*;
    use crate::peg_parser::parser_result::ParseTree;
    use crate::peg_parser::parser_token::{TokenValue, TokenType, Token};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TV {
        A,
        B,
        C,
    }

    impl TokenValue for TV {}

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TT {
        A,
        B,
        C,
    }

    impl TokenType for TT {}

    impl Token<TT, TV> for TV {
        fn to_val(&self) -> TV {
            *self
        }

        fn to_type(&self) -> TT {
            match self {
                TV::A => TT::A,
                TV::B => TT::B,
                TV::C => TT::C,
            }
        }
    }

    #[test]
    fn test_literal_exact() {
        let mut rules = HashMap::new();
        rules.insert("S", NicePegRule::LiteralExact(TV::A));

        let rules_raw = nice_rules_to_peg(rules, "S");

        let input = &[TV::A];
        let mut parser = Parser::new(&rules_raw.0);
        let res = parser.parse(input, rules_raw.1).ok().unwrap();
        assert_eq!(res.result, ParseTree::Value(TV::A));

        let input = &[TV::B];
        let mut parser = Parser::new(&rules_raw.0);
        assert!(parser.parse(input, rules_raw.1).is_err());
    }

    #[test]
    fn test_literal_bind() {
        let mut rules = HashMap::new();
        rules.insert("S", NicePegRule::LiteralBind(TT::A));

        let rules_raw = nice_rules_to_peg(rules, "S");

        let input = &[TV::A];
        let mut parser = Parser::new(&rules_raw.0);
        let res = parser.parse(input, rules_raw.1).ok().unwrap();
        assert_eq!(res.result, ParseTree::Value(TV::A));

        let input = &[TV::B];
        let mut parser = Parser::new(&rules_raw.0);
        assert!(parser.parse(input, rules_raw.1).is_err());
    }

    #[test]
    fn test_seq() {
        let mut rules = HashMap::new();
        rules.insert("S", NicePegRule::Sequence(vec![NicePegRule::LiteralExact(TV::A), NicePegRule::LiteralExact(TV::B), NicePegRule::LiteralExact(TV::C)]));

        let rules_raw = nice_rules_to_peg(rules, "S");

        let input = &[TV::A, TV::B, TV::C];
        let mut parser = Parser::new(&rules_raw.0);
        let res = parser.parse(input, rules_raw.1).ok().unwrap();
        assert_eq!(res.result, ParseTree::Sequence(vec![ParseTree::Value(TV::A), ParseTree::Value(TV::B), ParseTree::Value(TV::C)]));

        let input = &[TV::A, TV::B, TV::A];
        let mut parser = Parser::new(&rules_raw.0);
        assert!(parser.parse(input, rules_raw.1).is_err());
    }

    #[test]
    fn test_choice() {
        let mut rules = HashMap::new();
        rules.insert("S", NicePegRule::ChooseFirst(vec![NicePegRule::LiteralExact(TV::B), NicePegRule::LiteralExact(TV::C)]));

        let rules_raw = nice_rules_to_peg(rules, "S");

        let input = &[TV::B];
        let mut parser = Parser::new(&rules_raw.0);
        let res = parser.parse(input, rules_raw.1).ok().unwrap();
        assert_eq!(res.result, ParseTree::ChooseFirst(0, Box::new(ParseTree::Value(TV::B))));

        let input = &[TV::C];
        let mut parser = Parser::new(&rules_raw.0);
        let res = parser.parse(input, rules_raw.1).ok().unwrap();
        assert_eq!(res.result, ParseTree::ChooseFirst(1, Box::new(ParseTree::Value(TV::C))));

        let input = &[TV::A];
        let mut parser = Parser::new(&rules_raw.0);
        assert!(parser.parse(input, rules_raw.1).is_err());
    }

    #[test]
    fn test_left_recursive() {
        use crate::peg_parser::parser_result::ParseTree::*;

        let mut rules = HashMap::new();
        rules.insert("S", NicePegRule::Sequence(vec![NicePegRule::Rule("X"), NicePegRule::LiteralExact(TV::C)]));
        rules.insert("X", NicePegRule::ChooseFirst(vec![NicePegRule::Rule("Y"), NicePegRule::Sequence(vec![])]));
        rules.insert("Y", NicePegRule::Sequence(vec![NicePegRule::Rule("X"), NicePegRule::LiteralExact(TV::A)]));

        let rules_raw = nice_rules_to_peg(rules, "S");

        let input = &[TV::A, TV::A, TV::A, TV::C];
        let mut parser = Parser::new(&rules_raw.0);
        let res = parser.parse(input, rules_raw.1).ok().unwrap();
        let exp = Sequence(vec![ChooseFirst(0, Box::new(Sequence(vec![ChooseFirst(0, Box::new(Sequence(vec![ChooseFirst(0, Box::new(Sequence(vec![ChooseFirst(1, Box::new(Sequence(vec![]))), Value(TV::A)]))), Value(TV::A)]))), Value(TV::A)]))), Value(TV::C)]);
        assert_eq!(res.result, exp);

        let input = &[TV::B, TV::C];
        let mut parser = Parser::new(&rules_raw.0);
        assert!(parser.parse(input, rules_raw.1).is_err());
    }

    #[test]
    fn test_right_recursive() {
        use crate::peg_parser::parser_result::ParseTree::*;

        let mut rules = HashMap::new();
        rules.insert("S", NicePegRule::Sequence(vec![NicePegRule::Rule("X"), NicePegRule::LiteralExact(TV::C)]));
        rules.insert("X", NicePegRule::ChooseFirst(vec![NicePegRule::Rule("Y"), NicePegRule::Sequence(vec![])]));
        rules.insert("Y", NicePegRule::Sequence(vec![NicePegRule::LiteralExact(TV::A), NicePegRule::Rule("X")]));

        let rules_raw = nice_rules_to_peg(rules, "S");

        let input = &[TV::A, TV::A, TV::A, TV::C];
        let mut parser = Parser::new(&rules_raw.0);
        let res = parser.parse(input, rules_raw.1).ok().unwrap();
        let exp = Sequence(vec![ChooseFirst(0, Box::new(Sequence(vec![Value(TV::A), ChooseFirst(0, Box::new(Sequence(vec![Value(TV::A), ChooseFirst(0, Box::new(Sequence(vec![Value(TV::A), ChooseFirst(1, Box::new(Sequence(vec![])))])))])))]))), Value(TV::C)]);
        assert_eq!(res.result, exp);

        let input = &[TV::B, TV::C];
        let mut parser = Parser::new(&rules_raw.0);
        assert!(parser.parse(input, rules_raw.1).is_err());
    }

}