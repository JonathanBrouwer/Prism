use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::grammar_file::GrammarFile;
use crate::parsable::parsed::Parsed;
use crate::parsable::void::Void;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::placeholder_store::{ParsedPlaceholder, PlaceholderStore};

#[allow(clippy::type_complexity)]
pub struct ParsableDyn<'arn, Env> {
    pub from_construct: fn(
        span: Span,
        constructor: &'arn str,
        args: &[Parsed<'arn>],
        allocs: Allocs<'arn>,
        src: &InputTable<'arn>,
        env: &mut Env,
    ) -> Parsed<'arn>,

    pub create_eval_ctx: fn(
        constructor: &'arn str,
        parent_ctx: Parsed<'arn>,
        arg_placeholders: &[ParsedPlaceholder],
        // Env
        allocs: Allocs<'arn>,
        src: &InputTable<'arn>,
        env: &mut Env,
    ) -> Vec<Parsed<'arn>>,

    pub eval_to_grammar: fn(
        v: Parsed<'arn>,
        eval_ctx: Parsed<'arn>,
        placeholders: &PlaceholderStore<'arn, Env>,
        // Env
        src: &InputTable<'arn>,
        env: &mut Env,
    ) -> &'arn GrammarFile<'arn>,
}

impl<Env> Clone for ParsableDyn<'_, Env> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Env> Copy for ParsableDyn<'_, Env> {}

impl<'arn, Env> ParsableDyn<'arn, Env> {
    pub fn new<P: Parsable<'arn, Env>>() -> Self {
        Self {
            from_construct: from_construct_dyn::<Env, P>,
            create_eval_ctx: create_eval_ctx_dyn::<Env, P>,
            eval_to_grammar: eval_to_grammar_dyn::<Env, P>,
        }
    }
}

fn from_construct_dyn<'arn, Env, P: Parsable<'arn, Env>>(
    span: Span,
    constructor: &'arn str,
    args: &[Parsed<'arn>],
    allocs: Allocs<'arn>,
    src: &InputTable<'arn>,
    env: &mut Env,
) -> Parsed<'arn> {
    Parsed::from_value(allocs.alloc(P::from_construct(span, constructor, args, allocs, src, env)))
}

fn create_eval_ctx_dyn<'arn, Env, P: Parsable<'arn, Env>>(
    constructor: &'arn str,
    parent_ctx: Parsed<'arn>,
    arg_placeholders: &[ParsedPlaceholder],
    // Env
    allocs: Allocs<'arn>,
    src: &InputTable<'arn>,
    env: &mut Env,
) -> Vec<Parsed<'arn>> {
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

fn eval_to_grammar_dyn<'arn, Env, P: Parsable<'arn, Env>>(
    v: Parsed<'arn>,
    eval_ctx: Parsed<'arn>,
    placeholders: &PlaceholderStore<'arn, Env>,
    // Env
    src: &InputTable<'arn>,
    env: &mut Env,
) -> &'arn GrammarFile<'arn> {
    let eval_ctx = if eval_ctx.try_into_value::<Void>().is_some() {
        P::EvalCtx::default()
    } else {
        *eval_ctx.into_value()
    };
    P::eval_to_grammar(v.into_value(), eval_ctx, placeholders, src, env)
}
