use crate::core::pos::Pos;
use crate::core::span::Span;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use ariadne::{Report, ReportKind};
use std::marker::PhantomData;

/// Empty error is an error type that keeps track of no data, meant to be performant.
#[derive(Clone)]
pub struct EmptyError<'grm>(PhantomData<&'grm str>);

impl<'grm> ParseError for EmptyError<'grm> {
    type L = ErrorLabel<'grm>;

    fn new(_: Span) -> Self {
        Self(PhantomData)
    }

    fn add_label_explicit(&mut self, _: Self::L) {}

    fn add_label_implicit(&mut self, _: Self::L) {}

    fn merge(self, _: Self) -> Self {
        Self(PhantomData)
    }

    fn set_end(&mut self, _: Pos) {}

    fn report(&self, _enable_debug: bool) -> Report<'static, Span> {
        Report::build(ReportKind::Error, (), 0)
            .with_message("Parsing error in this file")
            .with_help(
                "Parsing was run in fast-mode, rerun without fast-mode to get more error details",
            )
            .finish()
    }
}
