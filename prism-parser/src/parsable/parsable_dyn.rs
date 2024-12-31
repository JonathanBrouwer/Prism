use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable2, ParseResult};
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct ParsableDyn<'arn, 'grm, Env> {
    pub from_construct: fn(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
    ) -> Parsed<'arn, 'grm>,
    phantom_data: PhantomData<Env>,
}

impl<'arn, 'grm: 'arn, Env> ParsableDyn<'arn, 'grm, Env> {
    pub fn new<P: Parsable2<'arn, 'grm, Env>>() -> Self {
        Self {
            from_construct: P::from_construct_dyn,
            phantom_data: PhantomData,
        }
    }
}
