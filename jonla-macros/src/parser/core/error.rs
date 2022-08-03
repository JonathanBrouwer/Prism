pub mod empty_error;
pub mod set_error;
pub mod tree_error;

use crate::parser::core::span::Span;
use crate::parser::core::stream::Stream;
use std::cmp::Ordering;

pub trait ParseError: Sized + Clone {
    type L;

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
