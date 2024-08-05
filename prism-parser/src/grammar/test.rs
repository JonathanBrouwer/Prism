use std::collections::HashMap;
use crate::core::cache::{CacheKey, CacheVal, ParserCacheEntry};
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::error::ParseError;
use crate::rule_action::RuleAction;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RuleExpr<'grm, Action> {
    Action(Action),
    Test(&'grm RuleExpr<'grm, Action>)
}

pub struct ParserState<'arn, 'grm> {
    v: &'arn RuleExpr<'grm, RuleAction<'arn, 'grm>>
}


pub trait Parser<'arn, 'grm: 'arn, O> {
    fn parse(
        &self,
        state: &mut ParserState<'arn, 'grm>,
    );
}

pub fn map_parser<'a, 'arn: 'a, 'grm: 'arn, O, P>(
    p: impl Parser<'arn, 'grm, O> + 'a,
) -> impl Parser<'arn, 'grm, P> + 'a {
    move |state: &mut ParserState<'arn, 'grm>| {
        p.parse(state);
    }
}

impl<
    'arn,
    'grm: 'arn,
    O,
    T: Fn(&mut ParserState<'arn, 'grm>),
> Parser<'arn, 'grm, O> for T
{
    fn parse(
        &self,
        state: &mut ParserState<'arn, 'grm>,
    ) {
        self(state)
    }
}
