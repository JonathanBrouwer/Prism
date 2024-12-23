use serde::{Deserialize, Serialize};
use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::grammar::rule::Rule;
use crate::grammar::serde_leak::*;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use crate::result_match;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct GrammarFile<'arn, 'grm> {
    #[serde(borrow, with = "leak_slice")]
    pub rules: &'arn [Rule<'arn, 'grm>],
}

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for GrammarFile<'arn, 'grm> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
    ) -> Self {
        assert_eq!(constructor, "GrammarFile");
        GrammarFile {
            rules: allocs.alloc_extend(args[0].into_value::<ParsedList>().into_iter().map(|rule| *rule.into_value::<Rule>())),
        }
    }
}

