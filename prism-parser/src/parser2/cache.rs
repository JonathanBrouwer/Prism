use std::collections::HashMap;
use std::marker::PhantomData;
use crate::core::adaptive::{GrammarStateId, RuleId};
use crate::core::pos::Pos;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::parser2::PResult;

pub struct ParserCache<'arn, 'grm: 'arn, E: ParseError<L= ErrorLabel<'grm>>> {
    map: HashMap<CacheKey, PResult<E>>,
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

#[derive(Eq, PartialEq, Hash)]
pub struct CacheKey {
    pos: Pos,
    rule: RuleId,
    grammar: GrammarStateId,
}