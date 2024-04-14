pub mod aggregate_error;
pub mod empty_error;
pub mod error_printer;
pub mod set_error;
pub mod tree_error;

use crate::core::pos::Pos;
use crate::core::span::Span;
use ariadne::Report;
use std::cmp::Ordering;

pub trait ParseError: Sized + Clone {
    type L;

    fn new(span: Span) -> Self;
    fn add_label_explicit(&mut self, label: Self::L);
    fn add_label_implicit(&mut self, label: Self::L);
    fn merge(self, other: Self) -> Self;
    fn set_end(&mut self, end: Pos);
    fn report(&self, enable_debug: bool) -> Report<'static, Span>;
}

pub fn err_combine<E: ParseError>((xe, xs): (E, Pos), (ye, ys): (E, Pos)) -> (E, Pos) {
    match xs.cmp(&ys) {
        Ordering::Less => (ye, ys),
        Ordering::Equal => (xe.merge(ye), xs),
        Ordering::Greater => (xe, xs),
    }
}

pub fn err_combine_opt<E: ParseError>(
    x: Option<(E, Pos)>,
    y: Option<(E, Pos)>,
) -> Option<(E, Pos)> {
    match (x, y) {
        (Some(x), Some(y)) => Some(err_combine(x, y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}
