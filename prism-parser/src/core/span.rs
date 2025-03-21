use crate::core::input_table::InputTable;
use crate::core::pos::Pos;
use serde::{Deserialize, Serialize};
use std::ops::Index;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl Span {
    pub fn new(start: Pos, end: Pos) -> Self {
        Span { start, end }
    }
}

impl<'grm> Index<Span> for InputTable<'grm> {
    type Output = &'grm str;

    fn index(&self, index: Span) -> &Self::Output {
        todo!()
    }
}
