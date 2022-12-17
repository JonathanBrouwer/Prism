use crate::error::ParseError;
use crate::core::span::Span;
use std::marker::PhantomData;

/// Empty error is an error type that keeps track of no data, meant to be performant.
#[derive(Clone)]
pub struct EmptyError<L: Clone>(PhantomData<L>);

impl<L: Clone> ParseError for EmptyError<L> {
    type L = L;

    fn new(_: Span) -> Self {
        Self(PhantomData)
    }

    fn add_label_explicit(&mut self, _: Self::L) {}

    fn add_label_implicit(&mut self, _: Self::L) {}

    fn merge(self, _: Self) -> Self {
        Self(PhantomData)
    }

    fn set_end(&mut self, _: usize) {}
}
