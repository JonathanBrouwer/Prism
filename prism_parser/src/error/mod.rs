pub mod aggregate_error;
pub mod empty_error;
pub mod error_label;
pub mod set_error;
pub mod tree_error;

use prism_diags::Diag;
use prism_input::pos::Pos;
use prism_input::span::Span;
use std::cmp::Ordering;

pub trait ParseError: Sized + Clone {
    type L;

    fn new(pos: Pos) -> Self;
    fn add_label_explicit(&mut self, label: Self::L);
    fn add_label_implicit(&mut self, label: Self::L);
    fn merge(self, other: Self) -> Self;
    fn span(&self) -> Span;
    fn set_end(&mut self, end: Pos);
    fn diag(&self) -> Diag;
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
