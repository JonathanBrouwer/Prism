use ariadne::{Cache, Source};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::path::PathBuf;
use std::sync::{RwLock, RwLockReadGuard};

#[derive(Default)]
pub struct InputTable<'arn> {
    inner: RwLock<InputTableInner<'arn>>,
}

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
    pub fn push_file(&self, file: &'arn str, path: PathBuf) -> InputTableIndex {
        let mut inner = self.inner.write().unwrap();
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
