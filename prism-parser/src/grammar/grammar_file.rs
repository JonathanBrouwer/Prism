use std::sync::Arc;

use crate::core::allocs::alloc_extend;
use crate::core::input::Input;
use crate::core::span::Span;
use crate::grammar::rule::Rule;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParsableError};
use crate::parser::parsed_list::ParsedList;
use crate::parser::placeholder_store::PlaceholderStore;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GrammarFile {
    pub rules: Arc<[Arc<Rule>]>,
}

impl<Db> Parsable<Db> for GrammarFile {
    type EvalCtx = ();

    fn from_construct(_span: Span, constructor: &Input, args: &[Parsed], _env: &mut Db) -> Self {
        assert_eq!(constructor.as_str(), "GrammarFile");
        GrammarFile {
            rules: alloc_extend(
                args[0]
                    .value_ref::<ParsedList>()
                    .iter()
                    .map(|((), v)| v)
                    .map(|rule| rule.value_cloned::<Rule>()),
            ),
        }
    }

    fn error_fallback(_env: &mut Db, _span: Span) -> Self {
        Self {
            rules: Arc::new([]),
        }
    }

    fn eval_to_grammar(
        self: &Arc<Self>,
        _eval_ctx: &Self::EvalCtx,
        _placeholders: &PlaceholderStore<Db>,
        _env: &mut Db,
    ) -> Result<Arc<GrammarFile>, ParsableError> {
        Ok(self.clone())
    }
}
