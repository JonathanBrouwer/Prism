use crate::core::allocs::alloc_extend;
use crate::core::input::Input;
use crate::core::span::Span;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use std::sync::Arc;

#[derive(Clone)]
pub struct ActionResult {
    pub span: Span,
    pub constructor: Input,
    pub args: Arc<[Parsed]>,
}

impl<Db> Parsable<Db> for ActionResult {
    type EvalCtx = ();

    fn from_construct(span: Span, constructor: &Input, args: &[Parsed], _env: &mut Db) -> Self {
        Self {
            span,
            constructor: constructor.clone(),
            args: alloc_extend(args.iter().cloned()),
        }
    }

    fn error_fallback(_env: &mut Db, _span: Span) -> Self {
        Self {
            span: Span::test(),
            constructor: Input::from_const("[ERROR]"),
            args: Arc::new([]),
        }
    }
}
