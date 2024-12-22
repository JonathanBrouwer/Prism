use crate::core::cache::Allocs;
use crate::core::span::Span;
use parsed::Parsed;
use std::any::type_name;
use std::hash::Hasher;

pub mod action_result;
pub mod guid;
pub mod parsable_dyn;
pub mod parsed;
pub mod parsed_debug;
pub mod void;

pub trait Parsable<'arn, 'grm: 'arn>: Sized + Sync + Send + Copy + 'arn {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
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
    ) -> Parsed<'arn, 'grm> {
        allocs
            .alloc(Self::from_construct(span, constructor, args, allocs))
            .to_parsed()
    }

    fn to_parsed(&'arn self) -> Parsed<'arn, 'grm> {
        Parsed::from_value(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::parsable::Parsable;

    #[derive(Debug, Copy, Clone)]
    struct A;
    #[derive(Debug, Copy, Clone)]
    struct B;
    impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for A {}
    impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for B {}

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
