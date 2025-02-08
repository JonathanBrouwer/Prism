use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::void::Void;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::apply_action::ParsedPlaceholder;

pub struct ParsableDyn<'arn, 'grm, Env> {
    pub from_construct: fn(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut Env,
    ) -> Parsed<'arn, 'grm>,

    pub create_eval_ctx: fn(
        _constructor: &'grm str,
        _parent_ctx: Parsed<'arn, 'grm>,
        _arg_placeholders: &[ParsedPlaceholder],
        // Env
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Vec<Parsed<'arn, 'grm>>,
}

impl<'arn, 'grm, Env> Clone for ParsableDyn<'arn, 'grm, Env> {
    fn clone(&self) -> Self {
        Self {
            from_construct: self.from_construct,
            create_eval_ctx: self.create_eval_ctx,
        }
    }
}

impl<'arn, 'grm, Env> Copy for ParsableDyn<'arn, 'grm, Env> {}

impl<'arn, 'grm: 'arn, Env> ParsableDyn<'arn, 'grm, Env> {
    pub fn new<P: Parsable<'arn, 'grm, Env>>() -> Self {
        Self {
            from_construct: from_construct_dyn::<Env, P>,
            create_eval_ctx: create_eval_ctx_dyn::<Env, P>,
        }
    }
}

fn from_construct_dyn<'arn, 'grm: 'arn, Env, P: Parsable<'arn, 'grm, Env>>(
    span: Span,
    constructor: &'grm str,
    args: &[Parsed<'arn, 'grm>],
    allocs: Allocs<'arn>,
    src: &'grm str,
    env: &mut Env,
) -> Parsed<'arn, 'grm> {
    Parsed::from_value(allocs.alloc(P::from_construct(span, constructor, args, allocs, src, env)))
}

fn create_eval_ctx_dyn<'arn, 'grm: 'arn, Env, P: Parsable<'arn, 'grm, Env>>(
    constructor: &'grm str,
    parent_ctx: Parsed<'arn, 'grm>,
    arg_placeholders: &[ParsedPlaceholder],
    // Env
    allocs: Allocs<'arn>,
    src: &'grm str,
    env: &mut Env,
) -> Vec<Parsed<'arn, 'grm>> {
    let parent_ctx = match parent_ctx.try_into_value::<Void>() {
        Some(Void) => P::EvalCtx::default(),
        None => *parent_ctx.into_value(),
    };

    let res = P::create_eval_ctx(constructor, parent_ctx, arg_placeholders, allocs, src, env);

    res.map(|v| match v {
        None => Void.to_parsed(),
        Some(v) => allocs.alloc(v).to_parsed(),
    })
    .collect()
}
