use std::cmp::Ordering;
use std::hash::Hash;

use crate::parser::customizable_parser::ParseRule::Expect;
use std::collections::HashMap;

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
pub struct ParseSuccess<'a, TT: TokenType, T: Token<TT>> {
    pub(crate) result: (),
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

type ParseResult<'a, TT, T> = Result<ParseSuccess<'a, TT, T>, ParseError<'a, TT, T>>;

pub struct ParserCache<'a, TT: TokenType, T: Token<TT>> {
    cache: HashMap<(usize, usize) , ParseResult<'a, TT, T>>
}

impl<'a, TT: TokenType, T: Token<TT>> ParserCache<'a, TT, T> {
    pub fn new() -> ParserCache<'a, TT, T> {
        ParserCache{ cache: HashMap::with_capacity(256) }
    }
    pub fn with_capacity(len: usize) -> ParserCache<'a, TT, T> {
        ParserCache{ cache: HashMap::with_capacity(64 * len) }
    }
}

pub fn next<'a, TT: TokenType, T: Token<TT>>(input: &'a [T]) -> (&'a T, &'a [T]) {
    assert!(input.len() > 0);
    (&input[0], &input[1..])
}

pub fn parse<'a, TT: TokenType, T: Token<TT>>(input: &'a [T], rule: usize, glb_rules: &Vec<PegRule<TT, T>>, cache: &mut ParserCache<'a, TT, T>) -> ParseResult<'a, TT, T> {
    //Check if result is in cache
    let key = (input.len(), rule);
    if cache.cache.contains_key(&key) {
        return cache.cache.get(&key).unwrap().clone();
    }

    //Deal with left recursion
    let mut prev_best = usize::MAX;

    //TODO this will be thrown for purely recursive rules
    cache.cache.insert(key, Err(ParseError { on: &input[0], expect: vec![], inv_priority: 0 }));
    let res = parse_sub(input, rule, glb_rules, cache);
    cache.cache.insert(key, res.clone());
    res

    // loop {
    //     let res = parse_sub(input, rule, glb_rules, cache);
    //
    //     match res {
    //         Ok(v) => {
    //             //Did we do better?
    //             if v.rest.len() < prev_best {
    //                 prev_best = v.rest.len();
    //                 cache.cache.insert(key, Ok(v));
    //             } else {
    //                 assert_eq!(v.rest.len(), prev_best);
    //                 return Ok(v);
    //             }
    //         },
    //         Err(v) => {
    //             return Err(v)
    //         }
    //     }
    // }
}

pub fn parse_sub<'a, TT: TokenType, T: Token<TT>>(input: &'a [T], rule: usize, glb_rules: &Vec<PegRule<TT, T>>, cache: &mut ParserCache<'a, TT, T>) -> ParseResult<'a, TT, T> {
    let rule = &glb_rules[rule];
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
                match parse(rest, rule, glb_rules, cache) {
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
                match parse(input, rule, glb_rules, cache) {
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
                let res = parse(input, *rule, glb_rules, cache)?;
                rest = res.rest;
                best_error = combine_err(best_error, res.best_error);
            }

            //Do from minimum to maximum amount of times
            for _ in (min.unwrap_or(0))..(max.unwrap_or(usize::MAX)) {
                let res = match parse(rest, *rule, glb_rules, cache) {
                    Ok(v) => v,
                    Err(_) => return Ok(ParseSuccess { result: (), rest, best_error})
                };
                rest = res.rest;
                best_error = combine_err(best_error, res.best_error);
            }

            return Ok(ParseSuccess { result: (), rest, best_error})
        }
        PegRule::Option(rule) => {
            match parse(input, *rule, glb_rules, cache) {
                Ok(v) => Ok(v),
                Err(_) => return Ok(ParseSuccess { result: (), rest: input, best_error: None})
            }
        }
        PegRule::LookaheadPositive(rule) => {
            match parse(input, *rule, glb_rules, cache) {
                //TODO should we return best error?
                Ok(v) => Ok(ParseSuccess {result: v.result, best_error: None, rest: input}),
                Err(e) => return Err(e)
            }
        }
        PegRule::LookaheadNegative(rule) => {
            match parse(input, *rule, glb_rules, cache) {
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
        assert!(parse(input, 0, &rules, &mut ParserCache::new()).is_ok())
    }

    #[test]
    fn test_literal_exact_err() {
        let input = &[T::A];
        let rules = vec![
            PegRule::LiteralExact(T::B)
        ];
        assert!(parse(input, 0, &rules, &mut ParserCache::new()).is_err())
    }

    #[test]
    fn test_left_recursive() {
        let input = &[T::A, T::A, T::A, T::C];
        let rules = vec![
            PegRule::Sequence(vec![1, 4]), // S = XC
            PegRule::ChooseFirst(vec![2, 3]), // X = Y | e
            PegRule::Sequence(vec![1, 3]), // Y = XA
            PegRule::LiteralExact(T::A),
            PegRule::LiteralExact(T::C),
            PegRule::Sequence(vec![])
        ];
        let res = parse(input, 0, &rules, &mut ParserCache::new());
        assert!(res.is_ok())
    }

    #[test]
    fn test_right_recursive() {
        let input = &[T::A, T::A, T::A, T::C];
        let rules = vec![
            PegRule::Sequence(vec![1, 4]), // S = XC
            PegRule::ChooseFirst(vec![2, 5]), // X = Y | e
            PegRule::Sequence(vec![3, 1]), // Y = AX
            PegRule::LiteralExact(T::A),
            PegRule::LiteralExact(T::C),
            PegRule::Sequence(vec![])
        ];
        assert!(parse(input, 0, &rules, &mut ParserCache::new()).is_ok())
    }
}