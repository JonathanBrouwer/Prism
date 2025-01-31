use crate::parsable::ParseResult;

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct Guid(pub usize);

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for Guid {}
