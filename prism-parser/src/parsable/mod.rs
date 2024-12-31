use crate::core::cache::Allocs;
use crate::core::span::Span;
use parsed::Parsed;
use std::any::type_name;

pub mod action_result;
pub mod env_capture;
pub mod guid;
pub mod option;
pub mod parsable_dyn;
pub mod parsed;
pub mod parsed_debug;
pub mod void;

pub trait ParseResult<'arn, 'grm: 'arn>: Sized + Sync + Send + Copy + 'arn {
    fn to_parsed(&'arn self) -> Parsed<'arn, 'grm> {
        Parsed::from_value(self)
    }
}

pub trait Parsable2<'arn, 'grm: 'arn, Env: Copy>:
    ParseResult<'arn, 'grm> + Sized + Sync + Send + Copy + 'arn
{
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
    ) -> Self {
        panic!(
            "Cannot parse a {} from a {constructor} constructor",
            type_name::<Self>()
        )
    }

    fn from_construct_dyn(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
    ) -> Parsed<'arn, 'grm> {
        allocs
            .alloc(Self::from_construct(span, constructor, args, allocs, src))
            .to_parsed()
    }
}

#[cfg(test)]
mod tests {
    use crate::parsable::{Parsable2, ParseResult};

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
