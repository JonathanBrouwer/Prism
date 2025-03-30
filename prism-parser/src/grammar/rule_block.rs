use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::grammar::identifier::{Identifier, parse_identifier};
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct RuleBlock<'arn> {
    pub name: Identifier,
    pub adapt: bool,
    #[serde(borrow, with = "leak_slice")]
    pub constructors: &'arn [AnnotatedRuleExpr<'arn>],
}

impl ParseResult for RuleBlock<'_> {}
impl<'arn, Env> Parsable<'arn, Env> for RuleBlock<'arn> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed<'arn>],
        allocs: Allocs<'arn>,
        src: &InputTable<'arn>,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor.as_str(src), "Block");
        RuleBlock {
            name: parse_identifier(args[0]),
            adapt: args[1]
                .into_value::<ParsedList>()
                .into_iter()
                .next()
                .is_some(),
            constructors: allocs.alloc_extend(
                args[2]
                    .into_value::<ParsedList>()
                    .into_iter()
                    .map(|((), v)| v)
                    .map(|c| *c.into_value::<AnnotatedRuleExpr>()),
            ),
        }
    }
}
