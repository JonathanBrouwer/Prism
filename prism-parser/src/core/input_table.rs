use crate::META_GRAMMAR_STR;
use ariadne::{Cache, Source};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::path::PathBuf;
use std::sync::{RwLock, RwLockReadGuard};

pub struct InputTable<'arn> {
    inner: RwLock<InputTableInner<'arn>>,
}

impl<'arn> Default for InputTable<'arn> {
    fn default() -> Self {
        let s = Self {
            inner: Default::default(),
        };
        let meta_idx = s.get_or_push_file(META_GRAMMAR_STR, "$META_GRAMMAR$".into());
        assert_eq!(meta_idx, META_INPUT_INDEX);
        s
    }
}

pub const META_INPUT_INDEX: InputTableIndex = InputTableIndex(0);

#[derive(Default)]
pub struct InputTableInner<'arn> {
    files: Vec<InputTableEntry<'arn>>,
}

struct InputTableEntry<'arn> {
    input: &'arn str,
    path: PathBuf,
    source: Source<&'arn str>,
}

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct InputTableIndex(usize);

impl<'arn> InputTable<'arn> {
    pub fn get_or_push_file(&self, file: &'arn str, path: PathBuf) -> InputTableIndex {
        let mut inner = self.inner.write().unwrap();

        // If there is already a file with this path, don't load it again
        if let Some(prev) = inner.files.iter().position(|e| e.path == path) {
            return InputTableIndex(prev);
        }

        inner.files.push(InputTableEntry {
            input: file,
            source: Source::from(file),
            path,
        });
        InputTableIndex(inner.files.len() - 1)
    }

    pub fn get_str(&self, idx: InputTableIndex) -> &'arn str {
        self.inner.read().unwrap().files[idx.0].input
    }

    pub fn get_path(&self, idx: InputTableIndex) -> PathBuf {
        self.inner.read().unwrap().files[idx.0].path.clone()
    }

    pub fn inner(&self) -> RwLockReadGuard<InputTableInner<'arn>> {
        self.inner.read().unwrap()
    }
}

impl<'arn> Cache<InputTableIndex> for &InputTableInner<'arn> {
    type Storage = &'arn str;

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
