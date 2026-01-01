use crate::core::pos::Pos;
use crate::core::span::Span;
use ariadne::{Cache, Source};
use std::convert::Infallible;
use std::fmt::{Debug, Display};
use std::mem;
use std::path::PathBuf;
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
    source: Source<String>,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct InputTableIndex(usize);

impl InputTableIndex {
    pub fn value(self) -> usize {
        self.0
    }

    pub fn test() -> Self {
        Self(0)
    }
}

impl InputTableInner {
    pub fn get_or_push_file(&mut self, file: String, path: PathBuf) -> InputTableIndex {
        // If there is already a file with this path, don't load it again
        if let Some(prev) = self.files.iter().position(|e| e.path == path) {
            return InputTableIndex(prev);
        }

        self.files.push(InputTableEntry {
            source: Source::from(file),
            path,
        });
        InputTableIndex(self.files.len() - 1)
    }

    pub fn get_str(&self, idx: InputTableIndex) -> &str {
        let s = self.files[idx.0].source.text();

        // Safety: We never remove strings from the InputTable
        unsafe { mem::transmute(s) }
    }

    pub fn get_path(&self, idx: InputTableIndex) -> PathBuf {
        self.files[idx.0].path.clone()
    }

    pub fn update_file(&mut self, idx: InputTableIndex, new_content: String) {
        let file = &mut self.files[idx.0];
        file.source = Source::from(new_content);
    }

    pub fn remove(&mut self, idx: InputTableIndex) {
        let file = &mut self.files[idx.0];
        file.source = Source::from(String::new());
        file.path = "[CLOSED]".into();
    }

    pub fn end_of_file(&self, idx: InputTableIndex) -> Pos {
        Pos::start_of(idx) + self.get_str(idx).len()
    }

    pub fn slice(&self, span: Span) -> &str {
        let start = span.start_pos().idx_in_file();
        &self.get_str(span.start_pos().file())[start..start + span.len()]
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

impl Cache<InputTableIndex> for &InputTableInner {
    type Storage = String;

    fn fetch(&mut self, idx: &InputTableIndex) -> Result<&Source<Self::Storage>, impl Debug> {
        Result::<_, Infallible>::Ok(&self.files[idx.0].source)
    }

    fn display<'a>(&self, idx: &'a InputTableIndex) -> Option<impl Display + 'a> {
        Some(Box::new(
            self.files[idx.0]
                .path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        ))
    }
}
