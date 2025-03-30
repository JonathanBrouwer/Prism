use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::identifier::Identifier;
use crate::grammar::rule_annotation::RuleAnnotation;
use crate::grammar::rule_expr::RuleExpr;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct AnnotatedRuleExpr<'arn> {
    #[serde(borrow, with = "leak_slice")]
    pub annotations: &'arn [RuleAnnotation],
    #[serde(borrow, with = "leak")]
    pub expr: &'arn RuleExpr<'arn>,
}

impl ParseResult for AnnotatedRuleExpr<'_> {}
impl<'arn, Env> Parsable<'arn, Env> for AnnotatedRuleExpr<'arn> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed<'arn>],
        allocs: Allocs<'arn>,
        input: &InputTable<'arn>,
        _env: &mut Env,
    ) -> Self {
        assert_eq!("AnnotatedExpr", constructor.as_str(input));
        Self {
            annotations: allocs.alloc_extend(
                args[0]
                    .into_value::<ParsedList>()
                    .into_iter()
                    .map(|((), v)| v)
                    .map(|annot| *annot.into_value::<RuleAnnotation>()),
            ),
            expr: args[1].into_value::<RuleExpr<'arn>>(),
        }
    }
}
