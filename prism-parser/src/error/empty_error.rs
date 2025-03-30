use crate::core::input_table::InputTable;
use crate::core::pos::Pos;
use crate::core::span::Span;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use ariadne::Report;
use std::marker::PhantomData;

/// Empty error is an error type that keeps track of no data, meant to be performant.
#[derive(Clone)]
pub struct EmptyError<'arn>(PhantomData<&'arn str>);

impl<'arn> ParseError for EmptyError<'arn> {
    type L = ErrorLabel;

    fn new(_: Pos) -> Self {
        Self(PhantomData)
    }

    fn add_label_explicit(&mut self, _: Self::L) {}

    fn add_label_implicit(&mut self, _: Self::L) {}

    fn merge(self, _: Self) -> Self {
        Self(PhantomData)
    }

    fn report(&self, _enable_debug: bool, _input: &InputTable) -> Report<'static, Span> {
        unreachable!()
    }
}
