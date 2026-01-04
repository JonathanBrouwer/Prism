use crate::pos::Pos;
use crate::span::Span;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Default)]
pub struct InputTable {
    inner: RwLock<InputTableInner>,
}

#[derive(Default, Clone)]
pub struct InputTableInner {
    files: Vec<InputTableEntry>,
}

#[derive(Clone)]
struct InputTableEntry {
    path: PathBuf,
    source: String,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct InputTableIndex(usize);

impl InputTableIndex {
    pub fn value(self) -> usize {
        self.0
    }

    pub fn dummy() -> Self {
        Self(0)
    }
}

impl InputTableInner {
    pub fn get_or_push_file(&mut self, file: String, path: PathBuf) -> InputTableIndex {
        // If there is already a file with this path, don't load it again
        if let Some(prev) = self.files.iter().position(|e| e.path == path) {
            return InputTableIndex(prev);
        }

        self.files.push(InputTableEntry { source: file, path });
        InputTableIndex(self.files.len() - 1)
    }

    pub fn get_str(&self, idx: InputTableIndex) -> &str {
        &self.files[idx.0].source
    }

    pub fn get_path(&self, idx: InputTableIndex) -> &Path {
        &self.files[idx.0].path
    }

    pub fn update_file(&mut self, idx: InputTableIndex, new_content: String) {
        let file = &mut self.files[idx.0];
        file.source = new_content;
    }

    pub fn remove(&mut self, idx: InputTableIndex) {
        let file = &mut self.files[idx.0];
        file.source = String::new();
        file.path = "[CLOSED]".into();
    }

    pub fn start_of(&self, idx: InputTableIndex) -> Pos {
        Pos::start_of(idx)
    }

    pub fn end_of(&self, idx: InputTableIndex) -> Pos {
        Pos::start_of(idx) + self.get_str(idx).len()
    }

    pub fn span_of(&self, idx: InputTableIndex) -> Span {
        Span::new_with_end(self.start_of(idx), self.end_of(idx))
    }

    pub fn slice(&self, span: Span) -> &str {
        let start = span.start_pos().idx_in_file();
        &self.get_str(span.start_pos().file())[start..start + span.len()]
    }

    /// Returns (line, col) of the pos
    /// Both are 0-indexed
    /// Not very efficient...
    pub fn line_col_of(&self, pos: Pos) -> (usize, usize) {
        let input = self.get_str(pos.file());

        let line = input[0..pos.idx_in_file()]
            .chars()
            .filter(|c| *c == '\n')
            .count();

        let last_line_start = input[0..pos.idx_in_file()].rfind('\n').unwrap_or(0);
        let col = input[last_line_start..pos.idx_in_file()].len();
        (line, col)
    }
}

impl InputTable {
    pub fn deep_clone(&self) -> InputTable {
        InputTable {
            inner: RwLock::new(self.inner.read().unwrap().clone()),
        }
    }

    pub fn inner(&self) -> RwLockReadGuard<'_, InputTableInner> {
        self.inner.read().unwrap()
    }

    pub fn inner_mut(&self) -> RwLockWriteGuard<'_, InputTableInner> {
        self.inner.write().unwrap()
    }
}
