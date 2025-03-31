use std::sync::Arc;

use crate::core::allocs::alloc_extend;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::identifier::Identifier;
use crate::grammar::rule::Rule;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use crate::parser::placeholder_store::PlaceholderStore;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GrammarFile {
    pub rules: Arc<[Arc<Rule>]>,
}

impl<Env> Parsable<Env> for GrammarFile {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed],
        src: &InputTable,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor.as_str(src), "GrammarFile");
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
        _placeholders: &PlaceholderStore<Env>,
        _src: &InputTable,
        _env: &mut Env,
    ) -> Arc<GrammarFile> {
        self.clone()
    }
}
