use crate::core::allocs::alloc_extend;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::identifier::Identifier;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use std::sync::Arc;

#[derive(Clone)]
pub struct ActionResult {
    pub span: Span,
    pub constructor: Identifier,
    pub args: Arc<[Parsed]>,
}

impl<Env> Parsable<Env> for ActionResult {
    type EvalCtx = ();

    fn from_construct(
        span: Span,
        constructor: Identifier,
        args: &[Parsed],

        _src: &InputTable,
        _env: &mut Env,
    ) -> Self {
        Self {
            span,
            constructor,
            args: alloc_extend(args.iter().cloned()),
        }
    }
}
