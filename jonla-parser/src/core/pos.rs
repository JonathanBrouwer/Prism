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

    pub fn next(self, input: &str) -> (Self, Option<(Span, char)>) {
        match input[self.0..].chars().next() {
            None => (self, None),
            Some(c) => (
                Self(self.0 + c.len_utf8()),
                Some((Span::new(self, Self(self.0 + c.len_utf8())), c)),
            ),
        }
    }

    pub fn invalid() -> Self {
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

// ///
// #[derive(Clone, Copy)]
// pub struct Pos(&'grm str, usize);
//
// impl<'grm> Pos {
//     pub fn new(s: &'grm str) -> Self {
//         StringStream(s, 0)
//     }
//
//     pub fn pos(self) -> usize {
//         self.1
//     }
//
//     pub fn cmp(self, other: Self) -> Ordering {
//         self.1.cmp(&other.1)
//     }
//
//     pub fn span_to(self, other: Self) -> Span {
//         Span::new(self.1, other.1)
//     }
//
//     pub fn next(self) -> (Self, Option<(Span, char)>) {
//         match self.0[self.1..].chars().next() {
//             None => (self, None),
//             Some(c) => (
//                 StringStream(self.0, self.1 + c.len_utf8()),
//                 Some((Span::new(self.1, self.1 + c.len_utf8()), c)),
//             ),
//         }
//     }
//
//     pub fn span_rest(self) -> Span {
//         Span::new(self.1, self.0.len())
//     }
//
//     pub fn src(self) -> &'grm str {
//         self.0
//     }
//
//     pub fn with_pos(self, pos: usize) -> Self {
//         Self(self.0, pos)
//     }
// }
