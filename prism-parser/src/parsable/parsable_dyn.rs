use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::Parsable;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct ParsableDyn<'arn, 'grm, Env> {
    pub from_construct: fn(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut Env,
    ) -> Parsed<'arn, 'grm>,
}

impl<'arn, 'grm: 'arn, Env> ParsableDyn<'arn, 'grm, Env> {
    pub fn new<P: Parsable<'arn, 'grm, Env>>() -> Self {
        Self {
            from_construct: P::from_construct_dyn,
        }
    }
}
