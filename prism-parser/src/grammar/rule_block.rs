use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::grammar::from_action_result::parse_identifier;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct RuleBlock<'arn> {
    pub name: &'arn str,
    pub adapt: bool,
    #[serde(borrow, with = "leak_slice")]
    pub constructors: &'arn [AnnotatedRuleExpr<'arn>],
}

impl<'arn> ParseResult for RuleBlock<'arn> {}
impl<'arn, Env> Parsable<'arn, Env> for RuleBlock<'arn> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'arn str,
        _args: &[Parsed<'arn>],
        _allocs: Allocs<'arn>,
        _src: &InputTable<'arn>,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor, "Block");
        RuleBlock {
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
                    .map(|((), v)| v)
                    .map(|c| *c.into_value::<AnnotatedRuleExpr>()),
            ),
        }
    }
}
