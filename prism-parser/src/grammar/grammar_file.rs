use crate::core::allocs::Allocs;
use crate::core::span::Span;
use crate::grammar::rule::Rule;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use crate::parser::placeholder_store::PlaceholderStore;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct GrammarFile<'arn, 'grm> {
    #[serde(borrow, with = "leak_slice")]
    pub rules: &'arn [Rule<'arn, 'grm>],
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for GrammarFile<'arn, 'grm> {}
impl<'arn, 'grm: 'arn, Env> Parsable<'arn, 'grm, Env> for GrammarFile<'arn, 'grm> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor, "GrammarFile");
        GrammarFile {
            rules: _allocs.alloc_extend(
                _args[0]
                    .into_value::<ParsedList>()
                    .into_iter()
                    .map(|rule| *rule.into_value::<Rule>()),
            ),
        }
    }

    fn eval_to_grammar(
        &'arn self,
        _eval_ctx: Self::EvalCtx,
        _placeholders: &PlaceholderStore<'arn, 'grm, Env>,
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> &'arn GrammarFile<'arn, 'grm> {
        self
    }
}
