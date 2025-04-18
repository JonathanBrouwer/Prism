use crate::core::pos::Pos;
use crate::core::span::Span;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use ariadne::Report;

/// Empty error is an error type that keeps track of no data, meant to be performant.
#[derive(Clone)]
pub struct EmptyError;

impl ParseError for EmptyError {
    type L = ErrorLabel;

    fn new(_: Pos) -> Self {
        Self
    }

    fn add_label_explicit(&mut self, _: Self::L) {}

    fn add_label_implicit(&mut self, _: Self::L) {}

    fn merge(self, _: Self) -> Self {
        Self
    }

    fn span(&self) -> Span {
        unreachable!()
    }

    fn set_end(&mut self, _: Pos) {}

    fn report(&self) -> Report<'static, Span> {
        unreachable!()
    }
}
