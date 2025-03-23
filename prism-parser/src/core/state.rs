use crate::core::allocs::Allocs;
use crate::core::cache::{CacheKey, CacheVal, ParserCacheEntry};
use crate::core::input_table::InputTable;
use crate::core::pos::Pos;
use crate::error::ParseError;
use crate::parsable::parsable_dyn::ParsableDyn;
use crate::parser::placeholder_store::PlaceholderStore;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ParserState<'arn, Env, E: ParseError> {
    // Cache for parser_cache_recurse
    cache: HashMap<CacheKey, ParserCacheEntry<CacheVal<'arn, E>>>,
    cache_stack: Vec<CacheKey>,
    // For allocating things that might be in the result
    pub alloc: Allocs<'arn>,
    pub input: Arc<InputTable<'arn>>,
    // For generating guids
    pub guid_counter: usize,
    // For recovery
    pub recovery_points: HashMap<Pos, Pos>,

    pub parsables: HashMap<&'arn str, ParsableDyn<'arn, Env>>,
    pub placeholders: PlaceholderStore<'arn, Env>,
}

impl<'arn, Env, E: ParseError> ParserState<'arn, Env, E> {
    pub fn new(
        input: Arc<InputTable<'arn>>,
        alloc: Allocs<'arn>,
        parsables: HashMap<&'arn str, ParsableDyn<'arn, Env>>,
    ) -> Self {
        ParserState {
            cache: HashMap::new(),
            cache_stack: Vec::new(),
            alloc,
            input,
            guid_counter: 0,
            recovery_points: HashMap::new(),
            parsables,
            placeholders: Default::default(),
        }
    }

    pub(crate) fn cache_is_read(&self, key: CacheKey) -> Option<bool> {
        self.cache.get(&key).map(|v| v.read)
    }

    pub(crate) fn cache_get(&mut self, key: &CacheKey) -> Option<&CacheVal<'arn, E>> {
        if let Some(v) = self.cache.get_mut(key) {
            v.read = true;
            Some(&v.value)
        } else {
            None
        }
    }

    pub(crate) fn cache_insert(&mut self, key: CacheKey, value: CacheVal<'arn, E>) {
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
