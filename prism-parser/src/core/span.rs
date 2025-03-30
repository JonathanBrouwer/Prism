use crate::core::input_table::{InputTable, InputTableIndex};
use crate::core::pos::Pos;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Span {
    start: Pos,
    len: usize,
}

impl Span {
    pub fn new(start: Pos, len: usize) -> Self {
        Span { start, len }
    }

    pub fn start_pos(self) -> Pos {
        self.start
    }

    pub fn start_pos_ref(&self) -> &Pos {
        &self.start
    }

    pub fn len(self) -> usize {
        self.len
    }

    pub fn is_empty(self) -> bool {
        self.len == 0
    }

    pub fn end_pos(self) -> Pos {
        self.start + self.len
    }
}

impl<'arn> InputTable<'arn> {
    pub fn slice(&self, span: Span) -> &'arn str {
        let start = span.start.idx_in_file();
        &self.get_str(span.start.file())[start..start + span.len]
    }
}
