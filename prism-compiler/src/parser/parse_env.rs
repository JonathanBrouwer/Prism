use crate::lang::env::Env;
use crate::lang::UnionIndex;
use prism_parser::core::cache::Allocs;
use prism_parser::core::span::Span;
use prism_parser::parsable::parsed::Parsed;
use prism_parser::parsable::Parsable;

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for Env {
    fn from_construct(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
    ) -> Self {
    }
}
