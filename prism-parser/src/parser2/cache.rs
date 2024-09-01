use std::collections::HashMap;
use std::marker::PhantomData;
use by_address::ByAddress;
use crate::core::adaptive::{BlockState, GrammarState, GrammarStateId, RuleId};
use crate::core::pos::Pos;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::parser2::PResult;

pub struct ParserCache<'arn, 'grm: 'arn, E: ParseError<L= ErrorLabel<'grm>>> {
    map: HashMap<CacheKey<'arn, 'grm>, PResult<E>>,
    phantom: PhantomData<&'arn &'grm str>,
}

impl<'arn, 'grm: 'arn, E: ParseError<L= ErrorLabel<'grm>>> Default for ParserCache<'arn, 'grm, E> {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
            phantom: PhantomData
        }
    }
}

impl<'arn, 'grm: 'arn, E: ParseError<L= ErrorLabel<'grm>>> ParserCache<'arn, 'grm, E> {
    pub fn insert(&mut self, key: CacheKey<'arn, 'grm>, value: PResult<E>) {
        self.map.insert(key, value);
    }

    pub fn get(&mut self, key: &CacheKey<'arn, 'grm>) -> Option<&PResult<E>> {
        self.map.get(key)
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct CacheKey<'arn, 'grm: 'arn> {
    pos: Pos,
    block: ByAddress<&'arn BlockState<'arn, 'grm>>,
    /// It is important that `grammar` is a cache key since block states may be reused between grammar states
    grammar: ByAddress<&'arn GrammarState<'arn, 'grm>>,
}

impl<'arn, 'grm: 'arn> CacheKey<'arn, 'grm> {
    pub fn new(pos: Pos, block: &'arn BlockState<'arn, 'grm>, grammar: &'arn GrammarState<'arn, 'grm>) -> Self {
        Self {
            pos,
            block: ByAddress(block),
            grammar: ByAddress(grammar),
        }
    }
}