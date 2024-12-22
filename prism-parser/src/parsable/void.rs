use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::Parsable;

#[derive(Copy, Clone)]
pub struct Void;

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for Void {
    fn from_construct(
        _span: Span,
        _constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
    ) -> Self {
        Self
    }
}
