use crate::core::input_table::InputTable;
use crate::core::pos::Pos;
use serde::{Deserialize, Serialize};
use std::ops::Index;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: Pos,
    pub end: Pos, //TODO len
}

impl Span {
    pub fn new(start: Pos, end: Pos) -> Self {
        Span { start, end }
    }
}

impl<'grm> InputTable<'grm> {
    pub fn slice(&self, span: Span) -> &'grm str {
        &self.get_str(span.start.file())[span.start.idx_in_file()..span.end.idx_in_file()]
    }
}
