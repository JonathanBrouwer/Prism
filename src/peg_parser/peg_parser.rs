use std::hash::Hash;

use std::collections::{HashMap, VecDeque, HashSet};
use crate::peg_parser::parse_result::*;
use crate::peg_parser::grammar_analysis::{empty_analysis, analyse_grammar};

pub trait Token<TT: TokenType>: Sized + Copy + Eq + Hash {
    fn to_type(&self) -> TT;
}

pub trait TokenType: Sized + Copy + Eq + Hash {}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum PegRule<TT: TokenType, T: Token<TT>> {
    LiteralExact(T),
    LiteralBind(TT),

    Sequence(Vec<usize>),

    ChooseFirst(Vec<usize>),

    LookaheadPositive(usize),
    LookaheadNegative(usize),
}

struct ParserCacheEntry<'a, TT: TokenType, T: Token<TT>> {
    result: ParseResult<'a, TT, T>,
    seen: bool,
}

struct ParserCache<'a, TT: TokenType, T: Token<TT>> {
    cache: HashMap<(usize, usize) , ParserCacheEntry<'a, TT, T>>,
    cache_layers: VecDeque<HashSet<(usize, usize)>>
}

impl<'a, TT: TokenType, T: Token<TT>> ParserCache<'a, TT, T> {
    pub fn get(&mut self, key: &(usize, usize)) -> Option<&ParseResult<'a, TT, T>> {
        self.cache.get_mut(key).unwrap().seen = true;
        self.cache.get(key).map(|r| &r.result)
    }

    pub fn is_seen(&self, key: &(usize, usize)) -> Option<bool> {
        self.cache.get(key).map(|r| r.seen)
    }

    pub fn remove(&mut self, key: &(usize, usize)) -> Option<ParseResult<'a, TT, T>> {
        self.cache.remove(key).map(|r| r.result)
    }

    pub fn contains_key(&mut self, key: &(usize, usize)) -> bool {
        self.cache.contains_key(key)
    }

    pub fn insert(&mut self, key: (usize, usize), result: ParseResult<'a, TT, T>) {
        self.cache.insert(key, ParserCacheEntry { result, seen: false });
        self.cache_layers.back_mut().unwrap().insert(key);
    }

    pub fn layer_incr(&mut self) {
        self.cache_layers.push_back(HashSet::new());
    }

    pub fn layer_revert(&mut self) {
        for key in self.cache_layers.pop_back().unwrap() {
            self.cache.remove(&key);
        }
    }

    pub fn layer_commit(&mut self) {
        self.cache_layers.pop_back();
    }
}

pub struct Parser<'a, TT: TokenType, T: Token<TT>> {
    glb_rules: &'a Vec<PegRule<TT, T>>,

    cache: ParserCache<'a, TT, T>,
}

impl<'a, TT: TokenType, T: Token<TT>> Parser<'a, TT, T> {
    pub fn new(rules: &'a Vec<PegRule<TT, T>>) -> Self {
        let mut s = Parser {
            glb_rules: rules,
            cache: ParserCache { cache: HashMap::new(), cache_layers: VecDeque::new() },
        };
        s.cache.layer_incr();
        s
    }

    pub fn parse(&mut self, input: &'a [T], rule: usize) -> ParseResult<'a, TT, T> {
        //Check if result is in cache
        let key = (input.len(), rule);
        if self.cache.contains_key(&key) {
            return self.cache.get(&key).unwrap().clone();
        }

        self.cache.layer_incr();
        //TODO fix this error
        self.cache.insert(key, Err(ParseError { on: &input[0], expect: vec![], inv_priority: 0 }));

        let res0 = self.parse_sub(input, rule);

        //If there's no left recursion, return the result
        if !self.cache.is_seen(&key).unwrap() {
            self.cache.layer_commit();
            self.cache.insert(key, res0.clone());
            return res0;
        }

        //If there's left recursion, but no seed, return the result
        if res0.is_err() {
            self.cache.layer_commit();
            self.cache.insert(key, res0.clone());
            return res0;
        }

        self.cache.layer_revert();
        self.cache.layer_incr();
        self.cache.insert(key, res0.clone());
        let mut seed = res0.ok().unwrap();

        loop {
            match self.parse_sub(input, rule) {
                Ok(v) => {
                    //Did we do better?
                    if v.rest.len() < seed.rest.len() {
                        seed = v.clone();
                        self.cache.layer_revert();
                        self.cache.layer_incr();
                        self.cache.insert(key, Ok(v));
                    } else {
                        self.cache.layer_revert();
                        self.cache.insert(key, Ok(seed.clone()));
                        return Ok(seed);
                    }
                },
                Err(_) => {
                    unreachable!("TODO: Is this reachable?");
                    self.cache.layer_revert();
                    self.cache.insert(key, Ok(seed.clone()));
                    return Ok(seed);
                }
            }
        }
    }

    fn parse_sub(&mut self, input: &'a [T], rule: usize) -> ParseResult<'a, TT, T> {
        let rule = &self.glb_rules[rule];
        match rule {
            PegRule::LiteralExact(expect) => {
                let (token, rest) = if input.len() > 0 {
                    (&input[0], &input[1..])
                } else {
                    panic!("Somehow skipped passed EOF token!");
                };
                if *token == *expect {
                    Ok(ParseSuccess { result: (), best_error: None, rest })
                } else {
                    Err(ParseError { on: &input[0], expect: vec![Expected::LiteralExact(*expect)], inv_priority: input.len()})
                }
            }
            PegRule::LiteralBind(expect) => {
                let (token, rest) = if input.len() > 0 {
                    (&input[0], &input[1..])
                } else {
                    panic!("Somehow skipped passed EOF token!");
                };
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
                    match self.parse(rest, rule) {
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
                    match self.parse(input, rule) {
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
            PegRule::LookaheadPositive(rule) => {
                match self.parse(input, *rule) {
                    //TODO should we return best error?
                    Ok(v) => Ok(ParseSuccess {result: v.result, best_error: v.best_error, rest: input}),
                    Err(e) => return Err(e)
                }
            }
            PegRule::LookaheadNegative(rule) => {
                match self.parse(input, *rule) {
                    Ok(_) => Err(ParseError{ on: &input[0], expect: vec![], inv_priority: input.len()}),
                    Err(_) => Ok(ParseSuccess {result: (), best_error: None, rest: input})
                }
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::peg_parser::peg_parser::*;

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
        let mut parser = Parser::new(&rules);
        assert!(parser.parse(input, 0).is_ok());
    }

    #[test]
    fn test_literal_exact_err() {
        let input = &[T::A];
        let rules = vec![
            PegRule::LiteralExact(T::B)
        ];
        let mut parser = Parser::new(&rules);
        assert!(parser.parse(input, 0).is_err());
    }

    #[test]
    fn test_left_recursive1() {
        let input = &[T::A, T::A, T::A, T::C];
        let rules = vec![
            PegRule::Sequence(vec![1, 4]), // S = XC
            PegRule::ChooseFirst(vec![2, 5]), // X = Y | e
            PegRule::Sequence(vec![1, 3]), // Y = XA
            PegRule::LiteralExact(T::A),
            PegRule::LiteralExact(T::C),
            PegRule::Sequence(vec![])
        ];
        let mut parser = Parser::new(&rules);
        assert!(parser.parse(input, 0).is_ok());
    }

    #[test]
    fn test_left_recursive2() {
        let input = &[T::A, T::A, T::A, T::C];
        let rules = vec![
            PegRule::Sequence(vec![1, 4]), // S = XC
            PegRule::ChooseFirst(vec![2, 3]), // X = Y | A
            PegRule::Sequence(vec![1, 3]), // Y = XA
            PegRule::LiteralExact(T::A),
            PegRule::LiteralExact(T::C),
            PegRule::Sequence(vec![])
        ];
        let mut parser = Parser::new(&rules);
        assert!(parser.parse(input, 0).is_ok());
    }

    #[test]
    fn test_left_recursive3() {
        let input = &[T::A, T::B, T::A, T::C];
        let rules = vec![
            PegRule::Sequence(vec![1, 5]), // S = XC
            PegRule::ChooseFirst(vec![2, 3]), // X = Y | A
            PegRule::Sequence(vec![1, 4, 1]), // Y = XBX
            PegRule::LiteralExact(T::A),
            PegRule::LiteralExact(T::B),
            PegRule::LiteralExact(T::C),
        ];
        let mut parser = Parser::new(&rules);
        assert!(parser.parse(input, 0).is_ok());
    }

    #[test]
    fn test_left_recursive4() {
        let input = &[T::A, T::A, T::A, T::C];

        let rules = vec![
            PegRule::ChooseFirst(vec![1, 2, 8]), // S = XB | AS | e
            PegRule::Sequence(vec![3, 6]), // XB
            PegRule::Sequence(vec![5, 0]), // AS

            PegRule::ChooseFirst(vec![4, 5]), // X = XA / A
            PegRule::Sequence(vec![3, 5]), // XA

            PegRule::LiteralExact(T::A),
            PegRule::LiteralExact(T::B),
            PegRule::LiteralExact(T::C),
            PegRule::Sequence(vec![]),
            PegRule::Sequence(vec![0, 7]),
        ];
        let mut parser = Parser::new(&rules);
        assert!(parser.parse(&input[..], 0).is_ok());
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
        let mut parser = Parser::new(&rules);
        assert!(parser.parse(input, 0).is_ok());
    }
}