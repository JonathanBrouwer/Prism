use std::sync::Arc;

use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::grammar_file::GrammarFile;
use crate::grammar::identifier::Identifier;
use crate::parsable::Parsable;
use crate::parsable::parsed::{ArcExt, Parsed};
use crate::parsable::void::Void;
use crate::parser::placeholder_store::{ParsedPlaceholder, PlaceholderStore};

#[allow(clippy::type_complexity)]
pub struct ParsableDyn<Env> {
    pub from_construct: fn(
        span: Span,
        constructor: Identifier,
        args: &[Parsed],
        src: &InputTable,
        env: &mut Env,
    ) -> Parsed,

    pub create_eval_ctx: fn(
        constructor: Identifier,
        parent_ctx: &Parsed,
        arg_placeholders: &[ParsedPlaceholder],
        // Env
        src: &InputTable,
        env: &mut Env,
    ) -> Vec<Parsed>,

    pub eval_to_grammar: fn(
        v: &Parsed,
        eval_ctx: &Parsed,
        placeholders: &PlaceholderStore<Env>,
        // Env
        src: &InputTable,
        env: &mut Env,
    ) -> Arc<GrammarFile>,
}

impl<Env> Clone for ParsableDyn<Env> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Env> Copy for ParsableDyn<Env> {}

impl<Env> ParsableDyn<Env> {
    pub fn new<P: Parsable<Env>>() -> Self {
        Self {
            from_construct: from_construct_dyn::<Env, P>,
            create_eval_ctx: create_eval_ctx_dyn::<Env, P>,
            eval_to_grammar: eval_to_grammar_dyn::<Env, P>,
        }
    }
}

fn from_construct_dyn<Env, P: Parsable<Env>>(
    span: Span,
    constructor: Identifier,
    args: &[Parsed],

    src: &InputTable,
    env: &mut Env,
) -> Parsed {
    Arc::new(P::from_construct(span, constructor, args, src, env)).to_parsed()
}

fn create_eval_ctx_dyn<Env, P: Parsable<Env>>(
    constructor: Identifier,
    parent_ctx: &Parsed,
    arg_placeholders: &[ParsedPlaceholder],

    src: &InputTable,
    env: &mut Env,
) -> Vec<Parsed> {
    let parent_ctx: &P::EvalCtx = match parent_ctx.try_value_ref::<P::EvalCtx>() {
        Some(v) => v,
        None => &P::EvalCtx::default(),
    };

    let res = P::create_eval_ctx(constructor, parent_ctx, arg_placeholders, src, env);

    res.map(|v| match v {
        None => Arc::new(Void).to_parsed(),
        Some(v) => Arc::new(v).to_parsed(),
    })
    .collect()
}

fn eval_to_grammar_dyn<Env, P: Parsable<Env>>(
    v: &Parsed,
    eval_ctx: &Parsed,
    placeholders: &PlaceholderStore<Env>,
    // Env
    src: &InputTable,
    env: &mut Env,
) -> Arc<GrammarFile> {
    let eval_ctx = if eval_ctx.try_value_ref::<Void>().is_some() {
        &P::EvalCtx::default()
    } else {
        eval_ctx.value_ref()
    };
    P::eval_to_grammar(&v.value_cloned(), eval_ctx, placeholders, src, env)
}
