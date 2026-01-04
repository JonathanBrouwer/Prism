use crate::grammar::grammar_file::GrammarFile;
use crate::parsable::Parsable;
use crate::parsable::parsed::{ArcExt, Parsed};
use crate::parsable::void::Void;
use crate::parser::placeholder_store::{ParsedPlaceholder, PlaceholderStore};
use prism_input::input::Input;
use prism_input::input_table::InputTable;
use prism_input::span::Span;
use std::sync::Arc;

#[allow(clippy::type_complexity)]
pub struct ParsableDyn<Db> {
    pub from_construct: fn(
        span: Span,
        constructor: &Input,
        args: &[Parsed],
        env: &mut Db,
        input: &InputTable,
    ) -> Parsed,

    pub create_eval_ctx: fn(
        constructor: &Input,
        parent_ctx: &Parsed,
        arg_placeholders: &[ParsedPlaceholder],
        // Env
        src: &InputTable,
        env: &mut Db,
    ) -> Vec<Parsed>,

    pub eval_to_grammar: fn(
        v: &Parsed,
        eval_ctx: &Parsed,
        placeholders: &PlaceholderStore<Db>,
        // Env
        src: &InputTable,
        env: &mut Db,
    ) -> Arc<GrammarFile>,
}

impl<Db> Clone for ParsableDyn<Db> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Db> Copy for ParsableDyn<Db> {}

impl<Db> ParsableDyn<Db> {
    pub fn new<P: Parsable<Db>>() -> Self {
        Self {
            from_construct: from_construct_dyn::<Db, P>,
            create_eval_ctx: create_eval_ctx_dyn::<Db, P>,
            eval_to_grammar: eval_to_grammar_dyn::<Db, P>,
        }
    }
}

fn from_construct_dyn<Db, P: Parsable<Db>>(
    span: Span,
    constructor: &Input,
    args: &[Parsed],
    env: &mut Db,
    input: &InputTable,
) -> Parsed {
    Arc::new(P::from_construct(span, constructor, args, env, input)).to_parsed()
}

fn create_eval_ctx_dyn<Db, P: Parsable<Db>>(
    constructor: &Input,
    parent_ctx: &Parsed,
    arg_placeholders: &[ParsedPlaceholder],

    _src: &InputTable,
    env: &mut Db,
) -> Vec<Parsed> {
    let parent_ctx: &P::EvalCtx = match parent_ctx.try_value_ref::<P::EvalCtx>() {
        Some(v) => v,
        None => &P::EvalCtx::default(),
    };

    let res = P::create_eval_ctx(constructor, parent_ctx, arg_placeholders, env);

    res.map(|v| match v {
        None => Arc::new(Void).to_parsed(),
        Some(v) => Arc::new(v).to_parsed(),
    })
    .collect()
}

fn eval_to_grammar_dyn<Db, P: Parsable<Db>>(
    v: &Parsed,
    eval_ctx: &Parsed,
    placeholders: &PlaceholderStore<Db>,
    // Env
    _src: &InputTable,
    env: &mut Db,
) -> Arc<GrammarFile> {
    let eval_ctx = if eval_ctx.try_value_ref::<Void>().is_some() {
        &P::EvalCtx::default()
    } else {
        eval_ctx.value_ref()
    };
    P::eval_to_grammar(&v.value_cloned(), eval_ctx, placeholders, env)
}
