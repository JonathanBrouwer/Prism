use crate::parser::core::span::Span;
use std::cmp::Ordering;

#[derive(Clone, Copy)]
pub struct StringStream<'grm>(&'grm str, usize);

impl<'grm> StringStream<'grm> {
    pub fn new(s: &'grm str) -> Self {
        StringStream(s, 0)
    }

    pub fn pos(self) -> usize {
        self.1
    }

    pub fn cmp(self, other: Self) -> Ordering {
        self.1.cmp(&other.1)
    }

    pub fn span_to(self, other: Self) -> Span {
        Span::new(self.1, other.1)
    }

    pub fn next(self) -> (Self, Option<(Span, char)>) {
        match self.0[self.1..].chars().next() {
            None => (self, None),
            Some(c) => (
                StringStream(self.0, self.1 + c.len_utf8()),
                Some((Span::new(self.1, self.1 + c.len_utf8()), c)),
            ),
        }
    }

    pub fn span_rest(self) -> Span {
        Span::new(self.1, self.0.len())
    }
}
