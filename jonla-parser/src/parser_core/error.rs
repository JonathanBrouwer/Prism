pub mod empty_error;
pub mod set_error;
pub mod tree_error;

use crate::parser_core::span::Span;
use crate::parser_core::stream::StringStream;
use std::cmp::Ordering;

pub trait ParseError: Sized + Clone {
    type L;

    fn new(span: Span) -> Self;
    fn add_label_explicit(&mut self, label: Self::L);
    fn add_label_implicit(&mut self, label: Self::L);
    fn merge(self, other: Self) -> Self;
}

pub fn err_combine<'grm, E: ParseError>(
    (xe, xs): (E, StringStream<'grm>),
    (ye, ys): (E, StringStream<'grm>),
) -> (E, StringStream<'grm>) {
    match xs.cmp(ys) {
        Ordering::Less => (ye, ys),
        Ordering::Equal => (xe.merge(ye), xs),
        Ordering::Greater => (xe, xs),
    }
}

pub fn err_combine_opt<'grm, E: ParseError>(
    x: Option<(E, StringStream<'grm>)>,
    y: Option<(E, StringStream<'grm>)>,
) -> Option<(E, StringStream<'grm>)> {
    match (x, y) {
        (Some(x), Some(y)) => Some(err_combine(x, y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}