use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::parsed_mut::ParsedMut;
use crate::parsable::{ComplexParsable, SimpleParsable};
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct ParsableDyn<'arn, 'grm, Env> {
    pub build: fn(
        constructor: &'grm str,
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut Env,
    ) -> ParsedMut<'arn, 'grm>,

    pub arg: fn(
        s: &mut ParsedMut<'arn, 'grm>,
        arg: usize,
        value: Parsed<'arn, 'grm>,
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut Env,
    ),

    pub finish: fn(
        s: &mut ParsedMut<'arn, 'grm>,
        span: Span,
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut Env,
    ) -> Parsed<'arn, 'grm>,
}

impl<'arn, 'grm: 'arn, Env> ParsableDyn<'arn, 'grm, Env> {
    pub fn new<P: ComplexParsable<'arn, 'grm, Env>>() -> Self {
        Self {
            build: build_dyn::<Env, P>,
            arg: arg_dyn::<Env, P>,
            finish: finish_dyn::<Env, P>,
        }
    }
}

fn build_dyn<'arn, 'grm: 'arn, Env, P: ComplexParsable<'arn, 'grm, Env>>(
    constructor: &'grm str,
    allocs: Allocs<'arn>,
    src: &'grm str,
    env: &mut Env,
) -> ParsedMut<'arn, 'grm> {
    ParsedMut::from_value(allocs.alloc(P::build(constructor, allocs, src, env)))
}

fn arg_dyn<'arn, 'grm: 'arn, Env, P: ComplexParsable<'arn, 'grm, Env>>(
    s: &mut ParsedMut<'arn, 'grm>,
    arg: usize,
    value: Parsed<'arn, 'grm>,
    allocs: Allocs<'arn>,
    src: &'grm str,
    env: &mut Env,
) {
    let builder = s.into_value_mut::<P::Builder>();
    P::arg(builder, arg, value, allocs, src, env);
}

fn finish_dyn<'arn, 'grm: 'arn, Env, P: ComplexParsable<'arn, 'grm, Env>>(
    s: &mut ParsedMut<'arn, 'grm>,
    span: Span,
    allocs: Allocs<'arn>,
    src: &'grm str,
    env: &mut Env,
) -> Parsed<'arn, 'grm> {
    let builder = s.into_value_mut::<P::Builder>();
    let s = P::finish(builder, span, allocs, src, env);
    Parsed::from_value(allocs.alloc(s))
}
