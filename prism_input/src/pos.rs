use crate::input_table::{InputTable, InputTableIndex};
use crate::span::Span;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Pos(usize, InputTableIndex);

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.1 != other.1 {
            return None;
        }
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for Pos {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Pos {
    pub(crate) fn start_of(idx: InputTableIndex) -> Self {
        Self(0, idx)
    }

    pub fn file(self) -> InputTableIndex {
        self.1
    }

    pub fn file_ref(&self) -> &InputTableIndex {
        &self.1
    }

    pub fn idx_in_file(self) -> usize {
        self.0
    }

    pub fn span_to(self, other: Self) -> Span {
        Span::new(self, other - self)
    }

    pub fn next(self, input: &InputTable) -> (Self, Option<(Span, char)>) {
        match input.inner().get_str(self.1)[self.0..].chars().next() {
            None => (self, None),
            Some(c) => (
                Self(self.0 + c.len_utf8(), self.1),
                Some((Span::new(self, c.len_utf8()), c)),
            ),
        }
    }

    pub fn dummy() -> Self {
        Self(0, InputTableIndex::dummy())
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<usize> for Pos {
    type Output = Pos;

    fn add(self, rhs: usize) -> Self::Output {
        Pos(self.0 + rhs, self.1)
    }
}

impl Sub<Pos> for Pos {
    type Output = usize;

    fn sub(self, rhs: Pos) -> Self::Output {
        assert_eq!(self.1, rhs.1);
        self.0 - rhs.0
    }
}
