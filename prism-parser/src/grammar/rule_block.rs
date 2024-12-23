use serde::{Deserialize, Serialize};
use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::grammar::from_action_result::parse_identifier;
use crate::grammar::leak_slice;
use crate::grammar::rule_expr::RuleExpr;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use crate::result_match;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct RuleBlock<'arn, 'grm> {
    pub name: &'grm str,
    pub adapt: bool,
    #[serde(borrow, with = "leak_slice")]
    pub constructors: &'arn [AnnotatedRuleExpr<'arn, 'grm>],
}

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for RuleBlock<'arn, 'grm> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
    ) -> Self {
        assert_eq!(constructor, "Block");
        RuleBlock {
            name: parse_identifier(args[0], src),
            adapt: args[1].into_value::<ParsedList>().into_iter().next().is_some(),
            constructors: allocs.alloc_extend(args[2].into_value::<ParsedList>().into_iter().map(|c| *c.into_value::<AnnotatedRuleExpr>()))
        }
    }
}

