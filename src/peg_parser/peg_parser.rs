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
                    Ok(ParseSuccess { result: ParseTree::Value(input[0]), best_error: None, rest })
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
                    Ok(ParseSuccess { result: ParseTree::Value(input[0]), best_error: None, rest })
                } else {
                    Err(ParseError { on: &input[0], expect: vec![Expected::LiteralBind(*expect)], inv_priority: input.len() })
                }
            }
            PegRule::Sequence(rules) => {
                let mut rest = input;
                let mut best_error: Option<ParseError<'a, TT, T>> = None;
                let mut result = Vec::new();
                for &rule in rules {
                    match self.parse(rest, rule) {
                        Ok(suc) => {
                            result.push(suc.result);
                            rest = suc.rest;
                            best_error = combine_err(best_error, suc.best_error);
                        }
                        Err(fail) => {
                            best_error = combine_err(best_error, Some(fail));
                            return Err(best_error.unwrap());
                        }
                    }
                }
                Ok(ParseSuccess { result: ParseTree::Sequence(result), best_error, rest })
            }
            PegRule::ChooseFirst(rules) => {
                let mut best_error: Option<ParseError<'a, TT, T>> = None;
                for (ruleid, &rule) in rules.iter().enumerate() {
                    match self.parse(input, rule) {
                        Ok(suc) => {
                            best_error = combine_err(best_error, suc.best_error);
                            return Ok(ParseSuccess { result: ParseTree::ChooseFirst(ruleid, Box::new(suc.result)), rest: suc.rest, best_error });
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
                    Ok(v) => Ok(ParseSuccess {result: ParseTree::Empty, best_error: v.best_error, rest: input}),
                    Err(e) => return Err(e)
                }
            }
            PegRule::LookaheadNegative(rule) => {
                match self.parse(input, *rule) {
                    Ok(_) => Err(ParseError{ on: &input[0], expect: vec![], inv_priority: input.len()}),
                    Err(_) => Ok(ParseSuccess {result: ParseTree::Empty, best_error: None, rest: input})
                }
            }
        }
    }
}
