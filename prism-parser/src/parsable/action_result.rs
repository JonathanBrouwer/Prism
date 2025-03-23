use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};

#[derive(Copy, Clone)]
pub enum ActionResult<'arn> {
    Construct(Span, &'arn str, &'arn [Parsed<'arn>]),
}

impl ParseResult for ActionResult<'_> {}
impl<'arn, Env> Parsable<'arn, Env> for ActionResult<'arn> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'arn str,
        _args: &[Parsed<'arn>],
        _allocs: Allocs<'arn>,
        _src: &InputTable<'arn>,
        _env: &mut Env,
    ) -> Self {
        Self::Construct(
            _span,
            constructor,
            _allocs.alloc_extend(_args.iter().copied()),
        )
    }
}
