use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::grammar::from_action_result::parse_identifier;
use crate::grammar::rule_block::RuleBlock;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable2, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Rule<'arn, 'grm> {
    pub name: &'grm str,
    pub adapt: bool,
    #[serde(with = "leak_slice")]
    pub args: &'arn [(&'grm str, &'grm str)],
    #[serde(borrow, with = "leak_slice")]
    pub blocks: &'arn [RuleBlock<'arn, 'grm>],
    pub return_type: &'grm str,
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for Rule<'arn, 'grm> {}
impl<'arn, 'grm: 'arn, Env> Parsable2<'arn, 'grm, Env> for Rule<'arn, 'grm> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
    ) -> Self {
        assert_eq!(constructor, "Rule");

        Rule {
            name: parse_identifier(args[0], src),
            adapt: args[1]
                .into_value::<ParsedList>()
                .into_iter()
                .next()
                .is_some(),
            args: allocs.alloc_extend(
                args[2]
                    .into_value::<ParsedList>()
                    .into_iter()
                    .map(|n| ("ActionResult", parse_identifier(n, src))),
            ),
            blocks: allocs.alloc_extend(
                args[3]
                    .into_value::<ParsedList>()
                    .into_iter()
                    .map(|block| *block.into_value::<RuleBlock>()),
            ),
            return_type: "ActionResult",
        }
    }
}
