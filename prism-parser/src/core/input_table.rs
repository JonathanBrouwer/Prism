use crate::core::pos::Pos;
use ariadne::{Cache, Source};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::ops::Index;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Default)]
pub struct InputTable<'arn> {
    files: RwLock<Vec<&'arn str>>,
}

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct InputTableIndex(usize);

impl<'arn> InputTable<'arn> {
    pub fn push_file(&self, file: &'arn str) -> InputTableIndex {
        let mut files = self.files.write().unwrap();
        files.push(file);
        InputTableIndex(files.len() - 1)
    }

    pub fn get_str(&self, idx: InputTableIndex) -> &'arn str {
        self.files.read().unwrap()[idx.0]
    }
}

impl<'arn> Cache<InputTableIndex> for &InputTable<'arn> {
    type Storage = &'arn str;

    fn fetch(
        &mut self,
        id: &InputTableIndex,
    ) -> Result<&Source<Self::Storage>, Box<dyn Debug + '_>> {
        todo!()
    }

    fn display<'a>(&self, id: &'a InputTableIndex) -> Option<Box<dyn Display + 'a>> {
        todo!()
    }
}
