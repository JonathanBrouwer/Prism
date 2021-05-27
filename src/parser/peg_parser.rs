use std::cmp::Ordering;
use std::hash::Hash;

use crate::parser::customizable_parser::ParseRule::Expect;
use std::collections::{HashMap, VecDeque, HashSet};

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

struct ParserCache<'a, TT: TokenType, T: Token<TT>> {
    cache: HashMap<(usize, usize) , ParseResult<'a, TT, T>>,
    cache_layers: VecDeque<HashSet<(usize, usize)>>
}

impl<'a, TT: TokenType, T: Token<TT>> ParserCache<'a, TT, T> {
    pub fn get(&mut self, key: &(usize, usize)) -> Option<&ParseResult<'a, TT, T>> {
        self.cache.get(key)
    }

    pub fn remove(&mut self, key: &(usize, usize)) -> Option<ParseResult<'a, TT, T>> {
        self.cache.remove(key)
    }

    pub fn contains_key(&mut self, key: &(usize, usize)) -> bool {
        self.cache.contains_key(key)
    }

    pub fn insert(&mut self, key: (usize, usize), v: ParseResult<'a, TT, T>) {
        self.cache.insert(key, v);
        self.cache_layers.back_mut().unwrap().insert(key);
    }

    pub fn layer_incr(&mut self) {
        self.cache_layers.push_back(HashSet::new());
    }

    pub fn layer_decr(&mut self) {
        for key in self.cache_layers.pop_back().unwrap() {
            self.cache.remove(&key);
        }
    }
}

type ParseResult<'a, TT, T> = Result<ParseSuccess<'a, TT, T>, ParseError<'a, TT, T>>;

pub fn next<'a, TT: TokenType, T: Token<TT>>(input: &'a [T]) -> (&'a T, &'a [T]) {
    assert!(input.len() > 0);
    (&input[0], &input[1..])
}

pub struct Parser<'a, TT: TokenType, T: Token<TT>> {
    glb_rules: &'a Vec<PegRule<TT, T>>,

    cache: ParserCache<'a, TT, T>,

    can_be_empty: Vec<bool>
}

impl<'a, TT: TokenType, T: Token<TT>> Parser<'a, TT, T> {
    pub fn new(rules: &'a Vec<PegRule<TT, T>>) -> Self {
        let mut s = Parser {
            glb_rules: rules,
            cache: ParserCache { cache: HashMap::new(), cache_layers: VecDeque::new() },
            can_be_empty: Vec::with_capacity(rules.len())
        };
        s.cache.layer_incr();
        s
    }

    // fn can_be_empty(&mut self, rule: usize) -> bool {
    //     let rule = &self.glb_rules[rule];
    //     let res = match rule {
    //         PegRule::LiteralExact(_) => {}
    //         PegRule::LiteralBind(_) => {}
    //         PegRule::Sequence(_) => {}
    //         PegRule::ChooseFirst(_) => {}
    //         PegRule::Repeat(_, _, _) => {}
    //         PegRule::Option(_) => {}
    //         PegRule::LookaheadPositive(_) => {}
    //         PegRule::LookaheadNegative(_) => {}
    //     }
    //     self.can_be_empty[]
    // }

    pub fn parse(&mut self, input: &'a [T], rule: usize) -> ParseResult<'a, TT, T> {
        //Check if result is in cache
        let key = (input.len(), rule);
        if self.cache.contains_key(&key) {
            return self.cache.get(&key).unwrap().clone();
        }

        //Deal with left recursion
        let mut prev_best = usize::MAX;

        //TODO this will be thrown for purely recursive rules
        self.cache.layer_incr();
        self.cache.insert(key, Err(ParseError { on: &input[0], expect: vec![], inv_priority: 0 }));

        loop {
            let res = self.parse_sub(input, rule);

            match res {
                Ok(v) => {
                    //Did we do better?
                    if v.rest.len() < prev_best {
                        prev_best = v.rest.len();
                        self.cache.layer_decr();
                        self.cache.layer_incr();
                        self.cache.insert(key, Ok(v));
                    } else {
                        let prev_v = self.cache.remove(&key).unwrap();
                        self.cache.layer_decr();
                        self.cache.insert(key, prev_v.clone());
                        return prev_v;
                    }
                },
                Err(v) => {
                    //TODO what if next call errors
                    self.cache.layer_decr();
                    self.cache.insert(key, Err(v.clone()));
                    return Err(v)
                }
            }
        }
    }

    fn parse_sub(&mut self, input: &'a [T], rule: usize) -> ParseResult<'a, TT, T> {
        let rule = &self.glb_rules[rule];
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
            PegRule::Repeat(rule, min, max) => {
                let mut rest = input;
                let mut best_error: Option<ParseError<'a, TT, T>> = None;

                //Do minimum amount of times
                for _ in 0..(min.unwrap_or(0)) {
                    let res = self.parse(input, *rule)?;
                    rest = res.rest;
                    best_error = combine_err(best_error, res.best_error);
                }

                //Do from minimum to maximum amount of times
                for _ in (min.unwrap_or(0))..(max.unwrap_or(usize::MAX)) {
                    let res = match self.parse(rest, *rule) {
                        Ok(v) => v,
                        Err(_) => return Ok(ParseSuccess { result: (), rest, best_error})
                    };
                    rest = res.rest;
                    best_error = combine_err(best_error, res.best_error);
                }

                return Ok(ParseSuccess { result: (), rest, best_error})
            }
            PegRule::Option(rule) => {
                match self.parse(input, *rule) {
                    Ok(v) => Ok(v),
                    Err(_) => return Ok(ParseSuccess { result: (), rest: input, best_error: None})
                }
            }
            PegRule::LookaheadPositive(rule) => {
                match self.parse(input, *rule) {
                    //TODO should we return best error?
                    Ok(v) => Ok(ParseSuccess {result: v.result, best_error: None, rest: input}),
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
    fn test_left_recursive() {
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