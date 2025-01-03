use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::SimpleParsable;
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
    pub fn new<P: SimpleParsable<'arn, 'grm, Env>>() -> Self {
        Self {
            from_construct: from_construct_dyn::<Env, P>,
        }
    }
}

fn from_construct_dyn<'arn, 'grm: 'arn, Env, P: SimpleParsable<'arn, 'grm, Env>>(
    span: Span,
    constructor: &'grm str,
    args: &[Parsed<'arn, 'grm>],
    allocs: Allocs<'arn>,
    src: &'grm str,
    env: &mut Env,
) -> Parsed<'arn, 'grm> {
    allocs
        .alloc(P::from_construct(span, constructor, args, allocs, src, env))
        .to_parsed()
}
