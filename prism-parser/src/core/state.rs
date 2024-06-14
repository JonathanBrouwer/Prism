use std::collections::HashMap;
use crate::core::cache::{Allocs, CacheKey, CacheVal, ParserCacheEntry};
use crate::error::ParseError;

pub struct ParserState<'grm, 'arn, E: ParseError> {
    // Cache for parser_cache_recurse
    cache: HashMap<CacheKey<'grm, 'arn>, ParserCacheEntry<CacheVal<'grm, 'arn, E>>>,
    cache_stack: Vec<CacheKey<'grm, 'arn>>,
    // For allocating things that might be in the result
    pub alloc: Allocs<'arn, 'grm>,
    pub input: &'grm str,
    // For generating guids
    pub guid_counter: usize,
}

pub type PState<'arn, 'grm, E> = ParserState<'grm, 'arn, E>;

impl<'grm, 'arn, E: ParseError> ParserState<'grm, 'arn, E> {
    pub fn new(input: &'grm str, alloc: Allocs<'arn, 'grm>) -> Self {
        ParserState {
            cache: HashMap::new(),
            cache_stack: Vec::new(),
            alloc,
            input,
            guid_counter: 0,
        }
    }

    pub(crate) fn cache_is_read(&self, key: CacheKey<'grm, 'arn>) -> Option<bool> {
        self.cache.get(&key).map(|v| v.read)
    }

    pub(crate) fn cache_get(
        &mut self,
        key: &CacheKey<'grm, 'arn>,
    ) -> Option<&CacheVal<'grm, 'arn, E>> {
        if let Some(v) = self.cache.get_mut(key) {
            v.read = true;
            Some(&v.value)
        } else {
            None
        }
    }

    pub(crate) fn cache_insert(
        &mut self,
        key: CacheKey<'grm, 'arn>,
        value: CacheVal<'grm, 'arn, E>,
    ) {
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
