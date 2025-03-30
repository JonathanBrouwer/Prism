use crate::META_GRAMMAR_STR;
use ariadne::{Cache, Source};
use serde::{Deserialize, Deserializer, Serialize};
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

#[derive(Default, Clone)]
pub struct InputTableInner<'arn> {
    files: Vec<InputTableEntry<'arn>>,
}

#[derive(Clone)]
struct InputTableEntry<'arn> {
    input: &'arn str,
    path: PathBuf,
    source: Source<&'arn str>,
}

#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug, Serialize)]
pub struct InputTableIndex(#[serde(skip)] usize);

#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for InputTableIndex {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[doc(hidden)]
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<InputTableIndex>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            #[automatically_derived]
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = InputTableIndex;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(
                        __formatter,
                        "tuple struct InputTableIndex",
                    )
                }
                #[inline]
                fn visit_newtype_struct<__E>(
                    self,
                    __e: __E,
                ) -> _serde::__private::Result<Self::Value, __E::Error>
                where
                    __E: _serde::Deserializer<'de>,
                {
                    let __field0: usize = <usize as _serde::Deserialize>::deserialize(__e)?;
                    _serde::__private::Ok(InputTableIndex(__field0))
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 = match _serde::de::SeqAccess::next_element::<usize>(&mut __seq)? {
                        _serde::__private::Some(__value) => __value,
                        _serde::__private::None => {
                            return _serde::__private::Err(_serde::de::Error::invalid_length(
                                0usize,
                                &"tuple struct InputTableIndex with 1 element",
                            ));
                        }
                    };
                    _serde::__private::Ok(InputTableIndex(__field0))
                }
            }
            let x = _serde::Deserializer::deserialize_newtype_struct(
                __deserializer,
                "InputTableIndex",
                __Visitor {
                    marker: _serde::__private::PhantomData::<InputTableIndex>,
                    lifetime: _serde::__private::PhantomData,
                },
            )?;
            Ok(Self(0))
        }
    }
};

impl<'arn> InputTable<'arn> {
    pub fn deep_clone<'brn>(&self) -> InputTable<'brn>
    where
        'arn: 'brn,
    {
        InputTable {
            inner: RwLock::new(self.inner.read().unwrap().clone()),
        }
    }

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
