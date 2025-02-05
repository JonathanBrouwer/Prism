use crate::core::cache::Allocs;
use crate::core::span::Span;
use parsed::Parsed;
use std::any::type_name;
use std::marker::PhantomData;

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

pub struct ParsedPlaceholder<'arn, 'grm>(usize, PhantomData<(&'arn (), &'grm ())>);

pub trait Parsable<'arn, 'grm: 'arn, Env>:
    ParseResult<'arn, 'grm> + Sized + Sync + Send + Copy + 'arn
{
    type EvalCtx: Default + Copy;

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
        &'arn self,
        _constructor: &'grm str,
        _parent_ctx: Self::EvalCtx,
        _arg_placeholders: &[ParsedPlaceholder<'arn, 'grm>],
        // Env
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Vec<Option<Self::EvalCtx>> {
        vec![]
    }

    fn eval_to_parsed(
        &'arn self,
        // Env
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> &'arn Self {
        self
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
