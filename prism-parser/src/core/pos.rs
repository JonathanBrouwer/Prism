use crate::core::input_table::META_INPUT_INDEX;
use crate::core::input_table::{InputTable, InputTableIndex};
use crate::core::span::Span;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};

fn meta_input_index() -> InputTableIndex {
    META_INPUT_INDEX
}

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Pos(
    usize,
    #[serde(skip, default = "meta_input_index")] InputTableIndex,
);

impl Pos {
    pub fn start_of(idx: InputTableIndex) -> Self {
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
        match input.get_str(self.1)[self.0..].chars().next() {
            None => (self, None),
            Some(c) => (
                Self(self.0 + c.len_utf8(), self.1),
                Some((Span::new(self, c.len_utf8()), c)),
            ),
        }
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
