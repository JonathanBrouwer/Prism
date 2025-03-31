use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::grammar_file::GrammarFile;
use crate::grammar::identifier::Identifier;
use crate::parser::placeholder_store::{ParsedPlaceholder, PlaceholderStore};
use parsed::Parsed;
use std::any::{Any, type_name};
use std::iter;
use std::sync::Arc;

pub mod action_result;
pub mod guid;
pub mod option;
pub mod parsable_dyn;
pub mod parsed;
pub mod parsed_debug;
pub mod void;

pub trait Parsable<Env>: Sized + Sync + Send + Any {
    type EvalCtx: Default + Clone + Send + Sync + Any;

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        _args: &[Parsed],
        // Env
        src: &InputTable,
        _env: &mut Env,
    ) -> Self {
        panic!(
            "Cannot parse a {} from a {} constructor",
            type_name::<Self>(),
            constructor.as_str(src)
        )
    }

    fn create_eval_ctx(
        _constructor: Identifier,
        _parent_ctx: &Self::EvalCtx,
        _arg_placeholders: &[ParsedPlaceholder],
        // Env
        _src: &InputTable,
        _env: &mut Env,
    ) -> impl Iterator<Item = Option<Self::EvalCtx>> {
        iter::empty()
    }

    fn eval_to_grammar(
        self: &Arc<Self>,
        _eval_ctx: &Self::EvalCtx,
        _placeholders: &PlaceholderStore<Env>,
        // Env
        _src: &InputTable,
        _env: &mut Env,
    ) -> Arc<GrammarFile> {
        unreachable!()
    }
}
