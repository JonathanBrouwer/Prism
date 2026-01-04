use crate::core::allocs::alloc_extend;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use prism_input::input::Input;
use prism_input::input_table::InputTable;
use prism_input::span::Span;
use std::sync::Arc;

#[derive(Clone)]
pub struct ActionResult {
    pub span: Span,
    pub constructor: Arc<String>,
    pub args: Arc<[Parsed]>,
}

impl<Db> Parsable<Db> for ActionResult {
    type EvalCtx = ();

    fn from_construct(
        span: Span,
        constructor: &Input,
        args: &[Parsed],
        _env: &mut Db,
        input: &InputTable,
    ) -> Self {
        Self {
            span,
            constructor: constructor.as_str(input).to_string().into(),
            args: alloc_extend(args.iter().cloned()),
        }
    }

    fn error_fallback(_env: &mut Db, _span: Span) -> Self {
        Self {
            span: Span::dummy(),
            constructor: "[ERROR]".to_string().into(),
            args: Arc::new([]),
        }
    }
}
