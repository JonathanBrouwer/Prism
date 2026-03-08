use crate::pos::Pos;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    start: Pos,
    len: usize,
}

impl Span {
    pub fn new(start: Pos, len: usize) -> Self {
        Span { start, len }
    }

    pub fn new_with_end(start: Pos, end: Pos) -> Self {
        assert_eq!(start.file(), end.file());
        Span {
            start,
            len: end.idx_in_file() - start.idx_in_file(),
        }
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

    pub fn dummy() -> Self {
        Self {
            start: Pos::dummy(),
            len: 0,
        }
    }

    pub fn span_to(self, other: Span) -> Span {
        self.span_to_pos(other.end_pos())
    }

    pub fn span_to_pos(self, other: Pos) -> Span {
        self.start_pos().span_to(other)
    }
}
