use crate::core::allocs::Allocs;
use crate::core::span::Span;
use crate::grammar::grammar_file::GrammarFile;
use crate::parser::placeholder_store::{ParsedPlaceholder, PlaceholderStore};
use parsed::Parsed;
use std::any::type_name;
use std::iter;

pub mod action_result;
pub mod env_capture;
pub mod guid;
pub mod option;
pub mod parsable_dyn;
pub mod parsed;
pub mod parsed_debug;
pub mod parsed_mut;
pub mod void;

pub trait ParseResult<'arn, 'grm: 'arn>: Sized + Sync + Send + Copy + 'arn {
    fn to_parsed(&'arn self) -> Parsed<'arn, 'grm> {
        Parsed::from_value(self)
    }
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for () {}

pub trait Parsable<'arn, 'grm: 'arn, Env>:
    ParseResult<'arn, 'grm> + Sized + Sync + Send + Copy + 'arn
{
    type EvalCtx: Default + Copy + ParseResult<'arn, 'grm>;

    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        // Env
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Self {
        panic!(
            "Cannot parse a {} from a {constructor} constructor",
            type_name::<Self>()
        )
    }

    fn create_eval_ctx(
        _constructor: &'grm str,
        _parent_ctx: Self::EvalCtx,
        _arg_placeholders: &[ParsedPlaceholder],
        // Env
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> impl Iterator<Item = Option<Self::EvalCtx>> {
        iter::empty()
    }

    fn eval_to_grammar(
        &'arn self,
        _eval_ctx: Self::EvalCtx,
        _placeholders: &PlaceholderStore<'arn, 'grm, Env>,
        // Env
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> &'arn GrammarFile<'arn, 'grm> {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use crate::parsable::ParseResult;

    #[derive(Debug, Copy, Clone)]
    struct A;
    #[derive(Debug, Copy, Clone)]
    struct B;
    impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for A {}
    impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for B {}

    #[test]
    fn a_a_same() {
        let a = A;
        let parsed = a.to_parsed();
        parsed.into_value::<A>();
    }

    #[test]
    #[should_panic]
    fn a_b_different() {
        let a = A;
        let parsed = a.to_parsed();
        parsed.into_value::<B>();
    }
}
