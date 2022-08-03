use crate::parser::core::span::Span;
use std::cmp::max;
use std::collections::HashSet;
use std::hash::Hash;
use std::marker::PhantomData;

pub trait ParseError: Sized {
    type L: Eq + Hash;

    fn new(span: Span) -> Self;
    fn add_label(&mut self, label: Self::L);
    fn merge(self, other: Self) -> Self;
}

#[derive(Clone, Debug)]
pub struct FullError<L: Eq + Hash> {
    span: Span,
    labels: HashSet<L>,
}

impl<L: Eq + Hash> ParseError for FullError<L> {
    type L = L;

    fn new(span: Span) -> Self {
        Self {
            span,
            labels: HashSet::new(),
        }
    }

    fn add_label(&mut self, label: L) {
        self.labels.insert(label);
    }

    fn merge(mut self, other: Self) -> Self {
        assert_eq!(self.span.start, other.span.start);
        for e in other.labels {
            self.labels.insert(e);
        }
        Self {
            span: Span::new(self.span.start, max(self.span.end, other.span.end)),
            labels: self.labels,
        }
    }
}

#[derive(Clone)]
pub struct EmptyError<L>(PhantomData<L>);

impl<L: Eq + Hash> ParseError for EmptyError<L> {
    type L = L;

    fn new(_: Span) -> Self {
        Self(PhantomData)
    }

    fn add_label(&mut self, _: Self::L) {}

    fn merge(self, _: Self) -> Self {
        Self(PhantomData)
    }
}
