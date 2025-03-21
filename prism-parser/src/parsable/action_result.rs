use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};

#[derive(Copy, Clone)]
pub enum ActionResult<'arn, 'grm> {
    Construct(Span, &'grm str, &'arn [Parsed<'arn, 'grm>]),
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for ActionResult<'arn, 'grm> {}
impl<'arn, 'grm: 'arn, Env> Parsable<'arn, 'grm, Env> for ActionResult<'arn, 'grm> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &InputTable<'grm>,
        _env: &mut Env,
    ) -> Self {
        Self::Construct(
            _span,
            constructor,
            _allocs.alloc_extend(_args.iter().copied()),
        )
    }
}
