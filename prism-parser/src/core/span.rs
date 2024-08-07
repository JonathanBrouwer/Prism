use crate::core::pos::Pos;
use serde::{Deserialize, Serialize};
use std::ops::Index;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl Span {
    pub fn new(start: Pos, end: Pos) -> Self {
        Span { start, end }
    }

    pub const fn invalid() -> Self {
        Span {
            start: Pos::invalid(),
            end: Pos::invalid(),
        }
    }
}

impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.start.into()..index.end.into()]
    }
}
