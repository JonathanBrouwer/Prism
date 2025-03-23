use crate::parsable::ParseResult;

#[derive(Copy, Clone)]
pub struct Void;

impl<'arn> ParseResult<'arn> for Void {}
