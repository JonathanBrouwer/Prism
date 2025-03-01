use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::grammar::grammar_file::GrammarFile;
use crate::parsable::parsed::Parsed;
use crate::parsable::void::Void;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::placeholder_store::{ParsedPlaceholder, PlaceholderStore};

#[allow(clippy::type_complexity)]
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
        constructor: &'grm str,
        parent_ctx: Parsed<'arn, 'grm>,
        arg_placeholders: &[ParsedPlaceholder],
        // Env
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut Env,
    ) -> Vec<Parsed<'arn, 'grm>>,

    pub eval_to_grammar: fn(
        v: Parsed<'arn, 'grm>,
        eval_ctx: Parsed<'arn, 'grm>,
        placeholders: &PlaceholderStore<'arn, 'grm, Env>,
        // Env
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut Env,
    ) -> &'arn GrammarFile<'arn, 'grm>,
}

impl<Env> Clone for ParsableDyn<'_, '_, Env> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Env> Copy for ParsableDyn<'_, '_, Env> {}

impl<'arn, 'grm: 'arn, Env> ParsableDyn<'arn, 'grm, Env> {
    pub fn new<P: Parsable<'arn, 'grm, Env>>() -> Self {
        Self {
            from_construct: from_construct_dyn::<Env, P>,
            create_eval_ctx: create_eval_ctx_dyn::<Env, P>,
            eval_to_grammar: eval_to_grammar_dyn::<Env, P>,
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
    let parent_ctx = match parent_ctx.try_into_value::<P::EvalCtx>() {
        Some(v) => *v,
        None => P::EvalCtx::default(),
    };

    let res = P::create_eval_ctx(constructor, parent_ctx, arg_placeholders, allocs, src, env);

    res.map(|v| match v {
        None => Void.to_parsed(),
        Some(v) => allocs.alloc(v).to_parsed(),
    })
    .collect()
}

fn eval_to_grammar_dyn<'arn, 'grm: 'arn, Env, P: Parsable<'arn, 'grm, Env>>(
    v: Parsed<'arn, 'grm>,
    eval_ctx: Parsed<'arn, 'grm>,
    placeholders: &PlaceholderStore<'arn, 'grm, Env>,
    // Env
    allocs: Allocs<'arn>,
    src: &'grm str,
    env: &mut Env,
) -> &'arn GrammarFile<'arn, 'grm> {
    let eval_ctx = if eval_ctx.try_into_value::<Void>().is_some() {
        P::EvalCtx::default()
    } else {
        *eval_ctx.into_value()
    };
    P::eval_to_grammar(v.into_value(), eval_ctx, placeholders, allocs, src, env)
}
