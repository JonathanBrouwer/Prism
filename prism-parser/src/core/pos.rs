use crate::core::span::Span;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Sub;

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Pos(usize);

impl Pos {
    pub fn start() -> Self {
        Self(0)
    }

    pub fn end(input: &str) -> Self {
        Self(input.len())
    }

    pub fn span_to(self, other: Self) -> Span {
        Span::new(self, other)
    }

    pub fn next(self, input: &str) -> (Span, Option<char>) {
        match input[self.0..].chars().next() {
            None => (self.span_to(self), None),
            Some(c) => (self.span_to(Self(self.0 + c.len_utf8())), Some(c)),
        }
    }

    pub const fn invalid() -> Self {
        Self(usize::MAX)
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Sub<Pos> for Pos {
    type Output = usize;

    fn sub(self, rhs: Pos) -> Self::Output {
        self.0 - rhs.0
    }
}

impl From<Pos> for usize {
    fn from(val: Pos) -> Self {
        val.0
    }
}
