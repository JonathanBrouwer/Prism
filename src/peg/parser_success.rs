use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use miette::{Diagnostic, GraphicalReportHandler, LabeledSpan, Severity, SourceCode};
use crate::peg::input::Input;
use crate::jonla::jerror::JError;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ParseSuccess<I: Input, O> {
    pub result: O,
    pub best_error: Option<JError<I>>,
    pub pos: I,
}

impl<I: Input, O> ParseSuccess<I, O> {
    pub fn map<F, ON>(self, mapfn: F) -> ParseSuccess<I, ON> where F: Fn(O) -> ON {
        ParseSuccess { result: mapfn(self.result), best_error: self.best_error, pos: self.pos }
    }
}