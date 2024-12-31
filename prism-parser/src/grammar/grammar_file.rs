use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::grammar::rule::Rule;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable2, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct GrammarFile<'arn, 'grm> {
    #[serde(borrow, with = "leak_slice")]
    pub rules: &'arn [Rule<'arn, 'grm>],
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for GrammarFile<'arn, 'grm> {}
impl<'arn, 'grm: 'arn, Env> Parsable2<'arn, 'grm, Env> for GrammarFile<'arn, 'grm> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        _src: &'grm str,
    ) -> Self {
        assert_eq!(constructor, "GrammarFile");
        GrammarFile {
            rules: allocs.alloc_extend(
                args[0]
                    .into_value::<ParsedList>()
                    .into_iter()
                    .map(|rule| *rule.into_value::<Rule>()),
            ),
        }
    }
}
