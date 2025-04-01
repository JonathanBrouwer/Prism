use ariadne::{Cache, Source};
use std::convert::Infallible;
use std::fmt::{Debug, Display};
use std::mem;
use std::path::PathBuf;
use std::sync::{RwLock, RwLockReadGuard};

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

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct InputTableIndex(usize);

impl InputTableIndex {
    pub fn value(self) -> usize {
        self.0
    }
}

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
