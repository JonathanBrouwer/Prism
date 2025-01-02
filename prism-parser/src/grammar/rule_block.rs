use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::grammar::from_action_result::parse_identifier;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct RuleBlock<'arn, 'grm> {
    pub name: &'grm str,
    pub adapt: bool,
    #[serde(borrow, with = "leak_slice")]
    pub constructors: &'arn [AnnotatedRuleExpr<'arn, 'grm>],
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for RuleBlock<'arn, 'grm> {}
impl<'arn, 'grm: 'arn, Env> Parsable<'arn, 'grm, Env> for RuleBlock<'arn, 'grm> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Result<Self, String> {
        assert_eq!(constructor, "Block");
        Ok(RuleBlock {
            name: parse_identifier(_args[0], _src),
            adapt: _args[1]
                .into_value::<ParsedList>()
                .into_iter()
                .next()
                .is_some(),
            constructors: _allocs.alloc_extend(
                _args[2]
                    .into_value::<ParsedList>()
                    .into_iter()
                    .map(|c| *c.into_value::<AnnotatedRuleExpr>()),
            ),
        })
    }
}
