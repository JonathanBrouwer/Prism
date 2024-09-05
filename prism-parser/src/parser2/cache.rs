use crate::core::adaptive::{BlockState, GrammarState};
use crate::core::pos::Pos;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::parser2::SequenceState;
use by_address::ByAddress;
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct ParserCache<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> {
    cache: HashMap<CacheKey<'arn, 'grm>, CacheEntry<'arn, 'grm, E>>,
    cache_stack: Vec<CacheKey<'arn, 'grm>>,
}

struct CacheEntry<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> {
    value: Result<SequenceState<'arn, 'grm>, E>,
    read: bool,
}

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> Default for ParserCache<'arn, 'grm, E> {
    fn default() -> Self {
        Self {
            cache: HashMap::default(),
            cache_stack: vec![],
        }
    }
}

#[derive(Copy, Clone)]
pub struct CacheState(usize);

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserCache<'arn, 'grm, E> {
    pub fn insert(
        &mut self,
        key: CacheKey<'arn, 'grm>,
        value: Result<SequenceState<'arn, 'grm>, E>,
    ) {
        self.cache.insert(key.clone(), CacheEntry {
            value,
            read: false,
        });
        self.cache_stack.push(key);
    }

    pub fn get(
        &mut self,
        key: &CacheKey<'arn, 'grm>,
    ) -> Option<&Result<SequenceState<'arn, 'grm>, E>> {
        let entry = self.cache.get_mut(key)?;
        entry.read = true;
        Some(&entry.value)
    }

    pub fn is_read(
        &self,
        key: &CacheKey<'arn, 'grm>,
    ) -> bool {
        self.cache.get(key).unwrap().read
    }

    pub fn cache_state_get(&self) -> CacheState {
        CacheState(self.cache_stack.len())
    }

    pub fn cache_state_revert(&mut self, CacheState(state): CacheState) {
        self.cache_stack.drain(state..).for_each(|key| {
            self.cache.remove(&key);
        })
    }
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct CacheKey<'arn, 'grm: 'arn> {
    pub pos: Pos,
    pub block: ByAddress<&'arn BlockState<'arn, 'grm>>,
    /// It is important that `grammar` is a cache key since block states may be reused between grammar states
    pub grammar: ByAddress<&'arn GrammarState<'arn, 'grm>>,
}

impl<'arn, 'grm: 'arn> CacheKey<'arn, 'grm> {
    pub fn new(
        pos: Pos,
        block: &'arn BlockState<'arn, 'grm>,
        grammar: &'arn GrammarState<'arn, 'grm>,
    ) -> Self {
        Self {
            pos,
            block: ByAddress(block),
            grammar: ByAddress(grammar),
        }
    }
}
