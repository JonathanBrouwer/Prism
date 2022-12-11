use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }
}
