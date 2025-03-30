use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::identifier::{Identifier, parse_identifier};
use crate::grammar::rule_block::RuleBlock;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Rule<'arn> {
    pub name: Identifier,
    pub adapt: bool,
    #[serde(borrow, with = "leak_slice")]
    pub args: &'arn [(Identifier, Identifier)],
    #[serde(borrow, with = "leak_slice")]
    pub blocks: &'arn [RuleBlock<'arn>],
    pub return_type: Identifier,
}

impl ParseResult for Rule<'_> {}
impl<'arn, Env> Parsable<'arn, Env> for Rule<'arn> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed<'arn>],
        allocs: Allocs<'arn>,
        src: &InputTable<'arn>,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor.as_str(src), "Rule");

        Rule {
            name: parse_identifier(args[0]),
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
                    .map(|n| (Identifier::from_const("ActionResult"), parse_identifier(n))),
            ),
            blocks: allocs.alloc_extend(
                args[3]
                    .into_value::<ParsedList>()
                    .into_iter()
                    .map(|((), v)| v)
                    .map(|block| *block.into_value::<RuleBlock>()),
            ),
            return_type: Identifier::from_const("ActionResult"),
        }
    }
}
