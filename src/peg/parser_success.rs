use std::cmp::Ordering;
use std::fmt::{Debug};
use crate::jonla::jerror::JError;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ParseSuccess<O> {
    pub result: O,
    pub best_error: Option<ParseError>,
    pub pos: usize,
}

impl<O> ParseSuccess<O> {
    pub fn map<F, ON>(self, mapfn: F) -> ParseSuccess<ON> where F: Fn(O) -> ON {
        ParseSuccess { result: mapfn(self.result), best_error: self.best_error, pos: self.pos }
    }
}

pub type ParseError = (JError, usize);

pub(crate) fn parse_error_combine_opt2(e1: Option<ParseError>, e2: Option<ParseError>) -> Option<ParseError> {
    match (e1, e2) {
        (Some(e1), Some(e2)) => Some(combine_or(e1, e2)),
        (Some(e1), None) => Some(e1),
        (None, Some(e2)) => Some(e2),
        (None, None) => None,
    }
}

pub(crate) fn parse_error_combine_opt1(e1: ParseError, e2: Option<ParseError>) -> ParseError {
    match e2 {
        Some(e2) => combine_or(e1, e2),
        None => e1
    }
}

pub fn combine_or(mut e1: ParseError, mut e2: ParseError) -> ParseError {
    match e1.1.cmp(&e2.1) {
        Ordering::Less => e2,
        Ordering::Greater => e1,
        Ordering::Equal => {
            e1.0.errors.append(&mut e2.0.errors);
            e1
        }
    }
}