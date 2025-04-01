use crate::core::input_table::InputTable;
use crate::core::pos::Pos;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
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

    pub fn test() -> Self {
        Self {
            start: Pos::test(),
            len: 0,
        }
    }
}

impl InputTable {
    pub fn slice(&self, span: Span) -> &str {
        let start = span.start.idx_in_file();
        &self.get_str(span.start.file())[start..start + span.len]
    }
}
