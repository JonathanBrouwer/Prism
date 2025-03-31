use crate::META_GRAMMAR_STR;
use ariadne::{Cache, Source};
use std::fmt::{Debug, Display};
use std::mem;
use std::path::PathBuf;
use std::sync::{RwLock, RwLockReadGuard};

pub struct InputTable {
    inner: RwLock<InputTableInner>,
}

impl Default for InputTable {
    fn default() -> Self {
        let s = Self {
            inner: Default::default(),
        };
        let meta_idx = s.get_or_push_file(META_GRAMMAR_STR.to_string(), "$META_GRAMMAR$".into());
        assert_eq!(meta_idx, META_INPUT_INDEX);
        s
    }
}

pub const META_INPUT_INDEX: InputTableIndex = InputTableIndex(0);

#[derive(Default, Clone)]
pub struct InputTableInner {
    files: Vec<InputTableEntry>,
}

#[derive(Clone)]
struct InputTableEntry {
    path: PathBuf,
    source: Source<String>,
}

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct InputTableIndex(usize);

impl InputTable {
    pub fn deep_clone(&self) -> InputTable {
        InputTable {
            inner: RwLock::new(self.inner.read().unwrap().clone()),
        }
    }

    pub fn get_or_push_file(&self, file: String, path: PathBuf) -> InputTableIndex {
        let mut inner = self.inner.write().unwrap();

        // If there is already a file with this path, don't load it again
        if let Some(prev) = inner.files.iter().position(|e| e.path == path) {
            return InputTableIndex(prev);
        }

        inner.files.push(InputTableEntry {
            source: Source::from(file),
            path,
        });
        InputTableIndex(inner.files.len() - 1)
    }

    pub fn get_str(&self, idx: InputTableIndex) -> &str {
        let lock = self.inner.read().unwrap();
        let s = lock.files[idx.0].source.text();

        // Safety: We never remove strings from the InputTable
        unsafe { mem::transmute(s) }
    }

    pub fn get_path(&self, idx: InputTableIndex) -> PathBuf {
        self.inner.read().unwrap().files[idx.0].path.clone()
    }

    pub fn inner(&self) -> RwLockReadGuard<InputTableInner> {
        self.inner.read().unwrap()
    }
}

impl Cache<InputTableIndex> for &InputTableInner {
    type Storage = String;

    fn fetch(
        &mut self,
        idx: &InputTableIndex,
    ) -> Result<&Source<Self::Storage>, Box<dyn Debug + '_>> {
        Ok(&self.files[idx.0].source)
    }

    fn display<'a>(&self, idx: &'a InputTableIndex) -> Option<Box<dyn Display + 'a>> {
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
