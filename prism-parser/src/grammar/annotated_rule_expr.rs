use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::grammar::rule_annotation::RuleAnnotation;
use crate::grammar::rule_expr::RuleExpr;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct AnnotatedRuleExpr<'arn, 'grm>(
    #[serde(borrow, with = "leak_slice")] pub &'arn [RuleAnnotation<'grm>],
    #[serde(borrow, with = "leak")] pub &'arn RuleExpr<'arn, 'grm>,
);

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for AnnotatedRuleExpr<'arn, 'grm> {}
impl<'arn, 'grm: 'arn, Env> Parsable<'arn, 'grm, Env> for AnnotatedRuleExpr<'arn, 'grm> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Result<Self, String> {
        assert_eq!("AnnotatedExpr", constructor);
        Ok(Self(
            _allocs.alloc_extend(
                _args[0]
                    .into_value::<ParsedList>()
                    .into_iter()
                    .map(|annot| *annot.into_value::<RuleAnnotation>()),
            ),
            _args[1].into_value::<RuleExpr<'arn, 'grm>>(),
        ))
    }
}
