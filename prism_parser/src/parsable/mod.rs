use crate::grammar::grammar_file::GrammarFile;
use crate::parser::placeholder_store::{ParsedPlaceholder, PlaceholderStore};
use parsed::Parsed;
use prism_input::input::Input;
use prism_input::input_table::InputTable;
use prism_input::span::Span;
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

pub trait Parsable<Db>: Sized + Sync + Send + Any {
    type EvalCtx: Default + Clone + Send + Sync + Any;

    fn from_construct(
        _span: Span,
        constructor: &Input,
        _args: &[Parsed],
        _env: &mut Db,
        input: &InputTable,
    ) -> Self {
        panic!(
            "Cannot parse a {} from a {} constructor",
            type_name::<Self>(),
            constructor.as_str(input)
        )
    }

    fn error_fallback(_env: &mut Db, _span: Span) -> Self;

    fn eval_to_grammar(
        self: &Arc<Self>,
        _eval_ctx: &Self::EvalCtx,
        _placeholders: &PlaceholderStore<Db>,
        _env: &mut Db,
    ) -> Arc<GrammarFile> {
        unreachable!()
    }

    fn create_eval_ctx(
        _constructor: &Input,
        _parent_ctx: &Self::EvalCtx,
        _arg_placeholders: &[ParsedPlaceholder],
        _env: &mut Db,
    ) -> impl Iterator<Item = Option<Self::EvalCtx>> {
        iter::empty()
    }
}
