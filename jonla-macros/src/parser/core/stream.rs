use crate::parser::core::span::Span;
use std::cmp::Ordering;

pub trait Stream: Sized + Clone + Copy {
    type I: Clone + Eq;

    fn pos(self) -> usize;
    fn cmp(self, other: Self) -> Ordering;
    fn span_to(self, other: Self) -> Span;
    fn next(self) -> (Self, Option<(Span, Self::I)>);
    fn span_rest(self) -> Span;
}

#[derive(Clone, Copy)]
pub struct StringStream<'a>(&'a str, usize);

impl Stream for StringStream<'_> {
    type I = char;

    fn pos(self) -> usize {
        self.1
    }

    fn cmp(self, other: Self) -> Ordering {
        self.1.cmp(&other.1)
    }

    fn span_to(self, other: Self) -> Span {
        Span::new(self.1, other.1)
    }

    fn next(self) -> (Self, Option<(Span, Self::I)>) {
        match self.0[self.1..].chars().next() {
            None => (self.clone(), None),
            Some(c) => (
                StringStream(self.0, self.1 + c.len_utf8()),
                Some((Span::new(self.1, self.1 + c.len_utf8()), c)),
            ),
        }
    }

    fn span_rest(self) -> Span {
        Span::new(self.1, self.0.len())
    }
}

impl<'a> Into<StringStream<'a>> for &'a str {
    fn into(self) -> StringStream<'a> {
        StringStream(self, 0)
    }
}
