use crate::parsable::ParseResult;

#[derive(Copy, Clone)]
pub struct Void;

impl ParseResult<'_> for Void {}
