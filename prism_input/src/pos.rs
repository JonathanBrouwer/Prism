use crate::input_table::{InputTable, InputTableIndex};
use crate::span::Span;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Pos {
    file: InputTableIndex,
    idx: usize,
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.file != other.file {
            return None;
        }
        Some(self.idx.cmp(&other.idx))
    }
}

impl Ord for Pos {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Pos {
    pub(crate) fn start_of(file: InputTableIndex) -> Self {
        Self { file, idx: 0 }
    }

    pub fn file(self) -> InputTableIndex {
        self.file
    }

    pub fn file_ref(&self) -> &InputTableIndex {
        &self.file
    }

    pub fn idx_in_file(self) -> usize {
        self.idx
    }

    pub fn span_to(self, other: Self) -> Span {
        Span::new(self, other - self)
    }

    pub fn next(self, input: &InputTable) -> Option<(char, Self)> {
        input.inner().get_str(self.file)[self.idx..]
            .chars()
            .next()
            .map(|c| (c, self + c.len_utf8()))
    }

    pub fn dummy() -> Self {
        Self {
            file: InputTableIndex::dummy(),
            idx: 0,
        }
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.idx)
    }
}

impl Add<usize> for Pos {
    type Output = Pos;

    fn add(self, rhs: usize) -> Self::Output {
        Pos {
            file: self.file,
            idx: self.idx + rhs,
        }
    }
}

impl Sub<Pos> for Pos {
    type Output = usize;

    fn sub(self, rhs: Pos) -> Self::Output {
        assert_eq!(self.file, rhs.file);
        self.idx - rhs.idx
    }
}
