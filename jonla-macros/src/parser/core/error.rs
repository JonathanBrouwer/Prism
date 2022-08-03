use crate::parser::core::span::Span;
use crate::parser::core::stream::Stream;
use std::cmp::{max, Ordering};
use std::collections::HashSet;
use std::hash::Hash;
use std::marker::PhantomData;

pub trait ParseError: Sized {
    type L: Eq + Hash;

    fn new(span: Span) -> Self;
    fn add_label(&mut self, label: Self::L);
    fn merge(self, other: Self) -> Self;
}

pub fn err_combine<E: ParseError, S: Stream>((xe, xs): (E, S), (ye, ys): (E, S)) -> (E, S) {
    match xs.cmp(ys) {
        Ordering::Less => (ye, ys),
        Ordering::Equal => (xe.merge(ye), xs),
        Ordering::Greater => (xe, xs),
    }
}

pub fn err_combine_opt<E: ParseError, S: Stream>(
    x: Option<(E, S)>,
    y: Option<(E, S)>,
) -> Option<(E, S)> {
    match (x, y) {
        (Some(x), Some(y)) => Some(err_combine(x, y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}

#[derive(Clone, Debug)]
pub struct FullError<L: Eq + Hash> {
    pub span: Span,
    pub labels: HashSet<L>,
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
