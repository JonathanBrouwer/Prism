use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::identifier::parse_identifier_old;
use crate::grammar::rule_block::RuleBlock;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct Rule<'arn> {
    pub name: &'arn str,
    pub adapt: bool,
    #[serde(with = "leak_slice")]
    pub args: &'arn [(&'arn str, &'arn str)],
    #[serde(borrow, with = "leak_slice")]
    pub blocks: &'arn [RuleBlock<'arn>],
    pub return_type: &'arn str,
}

impl ParseResult for Rule<'_> {}
impl<'arn, Env> Parsable<'arn, Env> for Rule<'arn> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'arn str,
        args: &[Parsed<'arn>],
        allocs: Allocs<'arn>,
        src: &InputTable<'arn>,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor, "Rule");

        Rule {
            name: parse_identifier_old(args[0], src),
            adapt: args[1]
                .into_value::<ParsedList>()
                .into_iter()
                .next()
                .is_some(),
            args: allocs.alloc_extend(
                args[2]
                    .into_value::<ParsedList>()
                    .into_iter()
                    .map(|((), v)| v)
                    .map(|n| ("ActionResult", parse_identifier_old(n, src))),
            ),
            blocks: allocs.alloc_extend(
                args[3]
                    .into_value::<ParsedList>()
                    .into_iter()
                    .map(|((), v)| v)
                    .map(|block| *block.into_value::<RuleBlock>()),
            ),
            return_type: "ActionResult",
        }
    }
}
