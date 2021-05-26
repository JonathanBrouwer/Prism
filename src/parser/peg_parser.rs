use std::cmp::Ordering;
use std::hash::Hash;

use crate::parser::customizable_parser::ParseRule::Expect;

pub trait Token<TT: TokenType>: Sized + Copy + Eq + Hash {
    fn to_type(&self) -> TT;
}

pub trait TokenType: Sized + Copy + Eq + Hash {}

#[derive(Eq, PartialEq, Clone)]
pub enum PegRule<TT: TokenType, T: Token<TT>> {
    LiteralExact(T),
    LiteralBind(TT),

    Sequence(Vec<usize>),

    ChooseFirst(Vec<usize>),
    // ChooseOne(Vec<&'a PegRule<'a, TT, T>>),

    Repeat(usize, Option<usize>, Option<usize>),
    Option(usize),

    LookaheadPositive(usize),
    LookaheadNegative(usize),
}

#[derive(Eq, PartialEq, Clone)]
pub struct ParseSuccess<'a, TT: TokenType, T: Token<TT>, R> {
    pub(crate) result: R,
    pub(crate) best_error: Option<ParseError<'a, TT, T>>,
    pub(crate) rest: &'a [T],
}

#[derive(Eq, PartialEq, Clone)]
pub enum Expected<TT: TokenType, T: Token<TT>> {
    LiteralExact(T),
    LiteralBind(TT),
}

#[derive(Eq, PartialEq, Clone)]
pub struct ParseError<'a, TT: TokenType, T: Token<TT>> {
    pub on: &'a T,
    pub expect: Vec<Expected<TT, T>>,
    pub inv_priority: usize
}

impl<'a, TT: TokenType, T: Token<TT>> ParseError<'a, TT, T> {
    pub fn combine(mut self, other: ParseError<'a, TT, T>) -> ParseError<'a, TT, T> {
        //TODO deal with ambiguous
        match self.inv_priority.cmp(&other.inv_priority) {
            // One has higher priority
            Ordering::Less => self,
            Ordering::Greater => other,
            // Equal priority
            Ordering::Equal => {
                for ex in other.expect {
                    if !self.expect.contains(&ex) {
                        self.expect.push(ex)
                    }
                }
                self
            }
        }
    }
}

pub fn combine_err<'a, TT: TokenType, T: Token<TT>>(a: Option<ParseError<'a, TT, T>>, b: Option<ParseError<'a, TT, T>>) -> Option<ParseError<'a, TT, T>> {
    match (a, b) {
        (None, None) => None,
        (Some(e), None) => Some(e),
        (None, Some(e)) => Some(e),
        (Some(e1), Some(e2)) => Some(e1.combine(e2))
    }
}

type ParseResult<'a, TT: TokenType, T: Token<TT>, R> = Result<ParseSuccess<'a, TT, T, R>, ParseError<'a, TT, T>>;

pub fn next<'a, TT: TokenType, T: Token<TT>>(input: &'a [T]) -> (&'a T, &'a [T]) {
    assert!(input.len() > 0);
    (&input[0], &input[1..])
}

pub fn parse<'a, TT: TokenType, T: Token<TT>>(input: &'a [T], rule: &PegRule<TT, T>, glb_rules: &Vec<PegRule<TT, T>>) -> ParseResult<'a, TT, T, ()> {
    match rule {
        PegRule::LiteralExact(expect) => {
            let (token, rest) = next(input);
            if *token == *expect {
                Ok(ParseSuccess { result: (), best_error: None, rest })
            } else {
                Err(ParseError { on: &input[0], expect: vec![Expected::LiteralExact(*expect)], inv_priority: input.len()})
            }
        }
        PegRule::LiteralBind(expect) => {
            let (token, rest) = next(input);
            if token.to_type() == *expect {
                Ok(ParseSuccess { result: (), best_error: None, rest })
            } else {
                Err(ParseError { on: &input[0], expect: vec![Expected::LiteralBind(*expect)], inv_priority: input.len() })
            }
        }
        PegRule::Sequence(rules) => {
            let mut rest = input;
            let mut best_error: Option<ParseError<'a, TT, T>> = None;
            for &rule in rules {
                match parse(rest, &glb_rules[rule], glb_rules) {
                    Ok(suc) => {
                        rest = suc.rest;
                        best_error = combine_err(best_error, suc.best_error);
                    }
                    Err(fail) => {
                        best_error = combine_err(best_error, Some(fail));
                        return Err(best_error.unwrap());
                    }
                }
            }
            Ok(ParseSuccess { result: (), best_error, rest })
        }
        PegRule::ChooseFirst(rules) => {
            assert!(rules.len() > 0);
            let mut best_error: Option<ParseError<'a, TT, T>> = None;
            for &rule in rules {
                match parse(input, &glb_rules[rule], glb_rules) {
                    Ok(suc) => {
                        best_error = combine_err(best_error, suc.best_error);
                        return Ok(ParseSuccess { result: suc.result, rest: suc.rest, best_error });
                    }
                    Err(fail) => {
                        best_error = combine_err(best_error, Some(fail));
                    }
                }
            }
            Err(best_error.unwrap())
        }
        PegRule::Repeat(rule, min, max) => {
            let mut rest = input;
            let mut best_error: Option<ParseError<'a, TT, T>> = None;

            //Do minimum amount of times
            for _ in 0..(min.unwrap_or(0)) {
                let res = parse(input, &glb_rules[*rule], glb_rules)?;
                rest = res.rest;
                best_error = combine_err(best_error, res.best_error);
            }

            //Do from minimum to maximum amount of times
            for _ in (min.unwrap_or(0))..(max.unwrap_or(usize::MAX)) {
                let res = match parse(rest, &glb_rules[*rule], glb_rules) {
                    Ok(v) => v,
                    Err(_) => return Ok(ParseSuccess { result: (), rest, best_error})
                };
                rest = res.rest;
                best_error = combine_err(best_error, res.best_error);
            }

            return Ok(ParseSuccess { result: (), rest, best_error})
        }
        PegRule::Option(rule) => {
            match parse(input, &glb_rules[*rule], glb_rules) {
                Ok(v) => Ok(v),
                Err(_) => return Ok(ParseSuccess { result: (), rest: input, best_error: None})
            }
        }
        PegRule::LookaheadPositive(rule) => {
            match parse(input, &glb_rules[*rule], glb_rules) {
                //TODO should we return best error?
                Ok(v) => Ok(ParseSuccess {result: v.result, best_error: None, rest: input}),
                Err(e) => return Err(e)
            }
        }
        PegRule::LookaheadNegative(rule) => {
            match parse(input, &glb_rules[*rule], glb_rules) {
                Ok(_) => Err(ParseError{ on: &input[0], expect: vec![], inv_priority: input.len()}),
                Err(_) => Ok(ParseSuccess {result: (), best_error: None, rest: input})
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::peg_parser::*;

    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    enum TT {
        A,
        B,
        C,
    }

    impl TokenType for TT {}

    #[test]
    fn test_literal_exact_ok() {
        let input = &[T::A];
        let rules = vec![
            PegRule::LiteralExact(T::A)
        ];
        assert!(parse(input, &rules[0], &rules).is_ok())
    }

    #[test]
    fn test_literal_exact_err() {
        let input = &[T::A];
        let rules = vec![
            PegRule::LiteralExact(T::B)
        ];
        assert!(parse(input, &rules[0], &rules).is_err())
    }

    #[test]
    fn test_left_recursive() {
        let input = &[T::A, T::A, T::A, T::C];
        let rules = vec![
            PegRule::Sequence(vec![1, 4]),
            PegRule::ChooseFirst(vec![2, 5]),
            PegRule::Sequence(vec![1, 3]),
            PegRule::LiteralExact(T::A),
            PegRule::LiteralExact(T::C),
            PegRule::Sequence(vec![])
        ];
        assert!(parse(input, &rules[0], &rules).is_ok())
    }

    #[test]
    fn test_right_recursive() {
        let input = &[T::A, T::A, T::A, T::C];
        let rules = vec![
            PegRule::Sequence(vec![1, 4]),
            PegRule::ChooseFirst(vec![2, 5]),
            PegRule::Sequence(vec![3, 1]),
            PegRule::LiteralExact(T::A),
            PegRule::LiteralExact(T::C),
            PegRule::Sequence(vec![])
        ];
        assert!(parse(input, &rules[0], &rules).is_ok())
    }
}