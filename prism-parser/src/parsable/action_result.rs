use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable2, ParseResult};

#[derive(Copy, Clone)]
pub enum ActionResult<'arn, 'grm> {
    Construct(Span, &'grm str, &'arn [Parsed<'arn, 'grm>]),
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for ActionResult<'arn, 'grm> {}
impl<'arn, 'grm: 'arn, Env: Copy> Parsable2<'arn, 'grm, Env> for ActionResult<'arn, 'grm> {
    fn from_construct(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        _src: &'grm str,
    ) -> Self {
        Self::Construct(span, constructor, allocs.alloc_extend(args.iter().copied()))
    }
}
