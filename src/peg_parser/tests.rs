#[cfg(test)]
mod tests {
    use crate::peg_parser::peg_parser::*;
    use std::collections::HashMap;
    use crate::peg_parser::nice_rules::*;
    use crate::peg_parser::parse_result::ParseTree;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum T {
        A,
        B,
        C,
    }

    impl Token<TT> for T {
        fn to_type(&self) -> TT {
            match self {
                T::A => TT::A,
                T::B => TT::B,
                T::C => TT::C,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TT {
        A,
        B,
        C,
    }

    impl TokenType for TT {}

    #[test]
    fn test_literal_exact() {
        let mut rules = HashMap::new();
        rules.insert("S", NicePegRule::LiteralExact(T::A));

        let rules_raw = nice_rules_to_peg(rules, "S");

        let input = &[T::A];
        let mut parser = Parser::new(&rules_raw.0);
        let res = parser.parse(input, rules_raw.1).ok().unwrap();
        assert_eq!(res.result, ParseTree::Value(T::A));

        let input = &[T::B];
        let mut parser = Parser::new(&rules_raw.0);
        assert!(parser.parse(input, rules_raw.1).is_err());
    }

    #[test]
    fn test_literal_bind() {
        let mut rules = HashMap::new();
        rules.insert("S", NicePegRule::LiteralBind(TT::A));

        let rules_raw = nice_rules_to_peg(rules, "S");

        let input = &[T::A];
        let mut parser = Parser::new(&rules_raw.0);
        let res = parser.parse(input, rules_raw.1).ok().unwrap();
        assert_eq!(res.result, ParseTree::Value(T::A));

        let input = &[T::B];
        let mut parser = Parser::new(&rules_raw.0);
        assert!(parser.parse(input, rules_raw.1).is_err());
    }

    #[test]
    fn test_seq() {
        let mut rules = HashMap::new();
        rules.insert("S", NicePegRule::Sequence(vec![NicePegRule::LiteralExact(T::A), NicePegRule::LiteralExact(T::B), NicePegRule::LiteralExact(T::C)]));

        let rules_raw = nice_rules_to_peg(rules, "S");

        let input = &[T::A, T::B, T::C];
        let mut parser = Parser::new(&rules_raw.0);
        let res = parser.parse(input, rules_raw.1).ok().unwrap();
        assert_eq!(res.result, ParseTree::Sequence(vec![ParseTree::Value(T::A), ParseTree::Value(T::B), ParseTree::Value(T::C)]));

        let input = &[T::A, T::B, T::A];
        let mut parser = Parser::new(&rules_raw.0);
        assert!(parser.parse(input, rules_raw.1).is_err());
    }

    #[test]
    fn test_choice() {
        let mut rules = HashMap::new();
        rules.insert("S", NicePegRule::ChooseFirst(vec![NicePegRule::LiteralExact(T::B), NicePegRule::LiteralExact(T::C)]));

        let rules_raw = nice_rules_to_peg(rules, "S");

        let input = &[T::B];
        let mut parser = Parser::new(&rules_raw.0);
        let res = parser.parse(input, rules_raw.1).ok().unwrap();
        assert_eq!(res.result, ParseTree::ChooseFirst(0, Box::new(ParseTree::Value(T::B))));

        let input = &[T::C];
        let mut parser = Parser::new(&rules_raw.0);
        let res = parser.parse(input, rules_raw.1).ok().unwrap();
        assert_eq!(res.result, ParseTree::ChooseFirst(1, Box::new(ParseTree::Value(T::C))));

        let input = &[T::A];
        let mut parser = Parser::new(&rules_raw.0);
        assert!(parser.parse(input, rules_raw.1).is_err());
    }

    #[test]
    fn test_left_recursive() {
        use crate::peg_parser::parse_result::ParseTree::*;

        let mut rules = HashMap::new();
        rules.insert("S", NicePegRule::Sequence(vec![NicePegRule::Rule("X"), NicePegRule::LiteralExact(T::C)]));
        rules.insert("X", NicePegRule::ChooseFirst(vec![NicePegRule::Rule("Y"), NicePegRule::Sequence(vec![])]));
        rules.insert("Y", NicePegRule::Sequence(vec![NicePegRule::Rule("X"), NicePegRule::LiteralExact(T::A)]));

        let rules_raw = nice_rules_to_peg(rules, "S");

        let input = &[T::A, T::A, T::A, T::C];
        let mut parser = Parser::new(&rules_raw.0);
        let res = parser.parse(input, rules_raw.1).ok().unwrap();
        let exp = Sequence(vec![ChooseFirst(0, Box::new(Sequence(vec![ChooseFirst(0, Box::new(Sequence(vec![ChooseFirst(0, Box::new(Sequence(vec![ChooseFirst(1, Box::new(Sequence(vec![]))), Value(T::A)]))), Value(T::A)]))), Value(T::A)]))), Value(T::C)]);
        assert_eq!(res.result, exp);

        let input = &[T::B, T::C];
        let mut parser = Parser::new(&rules_raw.0);
        assert!(parser.parse(input, rules_raw.1).is_err());
    }

    #[test]
    fn test_right_recursive() {
        use crate::peg_parser::parse_result::ParseTree::*;

        let mut rules = HashMap::new();
        rules.insert("S", NicePegRule::Sequence(vec![NicePegRule::Rule("X"), NicePegRule::LiteralExact(T::C)]));
        rules.insert("X", NicePegRule::ChooseFirst(vec![NicePegRule::Rule("Y"), NicePegRule::Sequence(vec![])]));
        rules.insert("Y", NicePegRule::Sequence(vec![NicePegRule::LiteralExact(T::A), NicePegRule::Rule("X")]));

        let rules_raw = nice_rules_to_peg(rules, "S");

        let input = &[T::A, T::A, T::A, T::C];
        let mut parser = Parser::new(&rules_raw.0);
        let res = parser.parse(input, rules_raw.1).ok().unwrap();
        let exp = Sequence(vec![ChooseFirst(0, Box::new(Sequence(vec![Value(T::A), ChooseFirst(0, Box::new(Sequence(vec![Value(T::A), ChooseFirst(0, Box::new(Sequence(vec![Value(T::A), ChooseFirst(1, Box::new(Sequence(vec![])))])))])))]))), Value(T::C)]);
        assert_eq!(res.result, exp);

        let input = &[T::B, T::C];
        let mut parser = Parser::new(&rules_raw.0);
        assert!(parser.parse(input, rules_raw.1).is_err());
    }

}