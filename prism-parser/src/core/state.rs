use crate::core::cache::{CacheKey, CacheVal, ParserCacheEntry};
use crate::core::input_table::InputTable;
use crate::error::ParseError;
use crate::parsable::parsable_dyn::ParsableDyn;
use crate::parser::placeholder_store::PlaceholderStore;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ParserState<Db, E: ParseError> {
    // Cache for parser_cache_recurse
    cache: HashMap<CacheKey, ParserCacheEntry<CacheVal<E>>>,
    cache_stack: Vec<CacheKey>,
    pub input: Arc<InputTable>,

    pub parsables: HashMap<&'static str, ParsableDyn<Db>>,
    pub placeholders: PlaceholderStore<Db>,
}

impl<Db, E: ParseError> ParserState<Db, E> {
    pub fn new(input: Arc<InputTable>, parsables: HashMap<&'static str, ParsableDyn<Db>>) -> Self {
        ParserState {
            cache: HashMap::new(),
            cache_stack: Vec::new(),
            input,
            parsables,
            placeholders: Default::default(),
        }
    }

    pub(crate) fn cache_is_read(&self, key: CacheKey) -> Option<bool> {
        self.cache.get(&key).map(|v| v.read)
    }

    pub(crate) fn cache_get(&mut self, key: &CacheKey) -> Option<&CacheVal<E>> {
        if let Some(v) = self.cache.get_mut(key) {
            v.read = true;
            Some(&v.value)
        } else {
            None
        }
    }

    pub(crate) fn cache_insert(&mut self, key: CacheKey, value: CacheVal<E>) {
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
}
