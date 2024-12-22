use crate::parsable::Parsable;

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Guid(pub usize);

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for Guid {}
