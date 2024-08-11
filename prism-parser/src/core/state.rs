use crate::core::cache::{Allocs, CacheKey, CacheVal, ParserCacheEntry};
use crate::core::pos::Pos;
use crate::error::ParseError;
use std::collections::HashMap;

pub struct ParserState<'arn, 'grm, E: ParseError> {
    // Cache for parser_cache_recurse
    cache: HashMap<CacheKey, ParserCacheEntry<CacheVal<'arn, 'grm, E>>>,
    cache_stack: Vec<CacheKey>,
    // For allocating things that might be in the result
    pub alloc: Allocs<'arn>,
    pub input: &'grm str,
    // For generating guids
    pub guid_counter: usize,
    // For recovery
    pub recovery_points: HashMap<Pos, Pos>,
}

impl<'arn, 'grm, E: ParseError> ParserState<'arn, 'grm, E> {
    pub fn new(input: &'grm str, alloc: Allocs<'arn>) -> Self {
        ParserState {
            cache: HashMap::new(),
            cache_stack: Vec::new(),
            alloc,
            input,
            guid_counter: 0,
            recovery_points: HashMap::new(),
        }
    }

    pub(crate) fn cache_is_read(&self, key: CacheKey) -> Option<bool> {
        self.cache.get(&key).map(|v| v.read)
    }

    pub(crate) fn cache_get(&mut self, key: &CacheKey) -> Option<&CacheVal<'arn, 'grm, E>> {
        if let Some(v) = self.cache.get_mut(key) {
            v.read = true;
            Some(&v.value)
        } else {
            None
        }
    }

    pub(crate) fn cache_insert(&mut self, key: CacheKey, value: CacheVal<'arn, 'grm, E>) {
        self.cache
            .insert(key.clone(), ParserCacheEntry { read: false, value });
        self.cache_stack.push(key);
    }

    pub(crate) fn cache_state_get(&self) -> usize {
        self.cache_stack.len()
    }

    pub(crate) fn cache_state_revert(&mut self, state: usize) {
        self.cache_stack.drain(state..).for_each(|key| {
            self.cache.remove(&key);
        })
    }

    pub(crate) fn clear(&mut self) {
        self.cache.clear();
        self.cache_stack.clear();
    }
}
