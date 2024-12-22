use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::Parsable;

pub struct ParsableDyn<'arn, 'grm> {
    pub from_construct: fn(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
    ) -> Parsed<'arn, 'grm>,
}

impl<'arn, 'grm: 'arn> ParsableDyn<'arn, 'grm> {
    pub fn new<P: Parsable<'arn, 'grm>>() -> Self {
        Self {
            from_construct: P::from_construct_dyn,
        }
    }
}
