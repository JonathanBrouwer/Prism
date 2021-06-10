use crate::peg_parser::parser_token::*;
use std::collections::{HashSet, VecDeque, HashMap};
use crate::peg_parser::parser_result::*;

pub struct ParserCacheEntry<'a, TT: TokenType, TV: TokenValue, T: Token<TT, TV>> {
    result: ParseResult<'a, TT, TV, T>,
    seen: bool,
}

pub struct ParserCache<'a, TT: TokenType, TV: TokenValue, T: Token<TT, TV>> {
    pub(crate) cache: HashMap<(usize, usize) , ParserCacheEntry<'a, TT, TV, T>>,
    pub(crate) cache_layers: VecDeque<HashSet<(usize, usize)>>
}

impl<'a, TT: TokenType, TV: TokenValue, T: Token<TT, TV>> ParserCache<'a, TT, TV, T> {
    pub fn get(&mut self, key: &(usize, usize)) -> Option<&ParseResult<'a, TT, TV, T>> {
        self.cache.get_mut(key).unwrap().seen = true;
        self.cache.get(key).map(|r| &r.result)
    }

    pub fn is_seen(&self, key: &(usize, usize)) -> Option<bool> {
        self.cache.get(key).map(|r| r.seen)
    }

    pub fn remove(&mut self, key: &(usize, usize)) -> Option<ParseResult<'a, TT, TV, T>> {
        self.cache.remove(key).map(|r| r.result)
    }

    pub fn contains_key(&mut self, key: &(usize, usize)) -> bool {
        self.cache.contains_key(key)
    }

    pub fn insert(&mut self, key: (usize, usize), result: ParseResult<'a, TT, TV, T>) {
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