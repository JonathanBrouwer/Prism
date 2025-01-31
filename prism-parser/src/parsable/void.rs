use crate::parsable::ParseResult;

#[derive(Copy, Clone)]
pub struct Void;

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for Void {}
