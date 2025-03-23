use crate::parsable::ParseResult;

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct Guid(pub usize);

impl ParseResult for Guid {}
