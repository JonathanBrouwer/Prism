use crate::parser_core::error::ParseError;
use crate::parser_core::span::Span;
use std::marker::PhantomData;

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
