use crate::core::input_table::InputTable;
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

    fn report(&self, _enable_debug: bool, _input: &InputTable) -> Report<'static, Span> {
        unreachable!()
    }
}
