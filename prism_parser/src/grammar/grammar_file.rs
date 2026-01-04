use std::sync::Arc;

use crate::core::allocs::alloc_extend;
use crate::grammar::rule::Rule;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use crate::parser::placeholder_store::PlaceholderStore;
use prism_input::input_table::InputTable;
use prism_input::span::Span;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GrammarFile {
    pub rules: Arc<[Arc<Rule>]>,
}

impl<Db> Parsable<Db> for GrammarFile {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &str,
        args: &[Parsed],
        _env: &mut Db,
        _input: &InputTable,
    ) -> Self {
        assert_eq!(constructor, "GrammarFile");
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

    fn eval_to_grammar(
        self: &Arc<Self>,
        _eval_ctx: &Self::EvalCtx,
        _placeholders: &PlaceholderStore<Db>,
        _env: &mut Db,
    ) -> Arc<GrammarFile> {
        self.clone()
    }
}
