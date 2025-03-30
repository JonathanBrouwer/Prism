use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::identifier::Identifier;
use crate::grammar::rule::Rule;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use crate::parser::placeholder_store::PlaceholderStore;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct GrammarFile<'arn> {
    #[serde(borrow, with = "leak_slice")]
    pub rules: &'arn [Rule<'arn>],
}

impl ParseResult for GrammarFile<'_> {}
impl<'arn, Env> Parsable<'arn, Env> for GrammarFile<'arn> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed<'arn>],
        allocs: Allocs<'arn>,
        src: &InputTable<'arn>,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor.as_str(src), "GrammarFile");
        GrammarFile {
            rules: allocs.alloc_extend(
                args[0]
                    .into_value::<ParsedList>()
                    .into_iter()
                    .map(|((), v)| v)
                    .map(|rule| *rule.into_value::<Rule>()),
            ),
        }
    }

    fn eval_to_grammar(
        &'arn self,
        _eval_ctx: Self::EvalCtx,
        _placeholders: &PlaceholderStore<'arn, Env>,
        _src: &InputTable<'arn>,
        _env: &mut Env,
    ) -> &'arn GrammarFile<'arn> {
        self
    }
}
