use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable2, ParseResult};

#[derive(Copy, Clone)]
pub struct Void;

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for Void {}
