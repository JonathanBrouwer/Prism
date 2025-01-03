use crate::parsable::parsed::checksum_parsable;
use crate::parsable::ParseResult;
use std::any::type_name;
use std::fmt::{Debug, Formatter};
use std::hash::{DefaultHasher, Hasher};
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;

#[derive(Clone)] //TODO unsound
pub struct ParsedMut<'arn, 'grm> {
    ptr: NonNull<()>,
    checksum: u64,
    pub(crate) name: &'static str,
    phantom_data: PhantomData<(&'arn (), &'grm ())>,
}

impl<'arn, 'grm: 'arn> ParsedMut<'arn, 'grm> {
    pub fn from_value<P: ParseResult<'arn, 'grm>>(p: &'arn mut P) -> Self {
        ParsedMut {
            ptr: NonNull::from(p).cast(),
            checksum: checksum_parsable::<P>(),
            name: type_name::<P>(),
            phantom_data: Default::default(),
        }
    }

    pub fn into_value<P: ParseResult<'arn, 'grm>>(self) -> &'arn mut P {
        let name = self.name;
        self.try_into_value().unwrap_or_else(|| {
            panic!(
                "Expected wrong king of Parsable. Expected {}, got {}",
                type_name::<P>(),
                name
            )
        })
    }

    pub fn into_value_mut<P: ParseResult<'arn, 'grm>>(&mut self) -> &mut &'arn mut P {
        let name = self.name;
        self.try_into_value_mut().unwrap_or_else(|| {
            panic!(
                "Expected wrong king of Parsable. Expected {}, got {}",
                type_name::<P>(),
                name
            )
        })
    }

    pub fn try_into_value<P: ParseResult<'arn, 'grm>>(self) -> Option<&'arn mut P> {
        if self.checksum != checksum_parsable::<P>() {
            return None;
        }
        Some(unsafe { self.ptr.cast::<P>().as_mut() })
    }

    pub fn try_into_value_mut<P: ParseResult<'arn, 'grm>>(&mut self) -> Option<&mut &'arn mut P> {
        if self.checksum != checksum_parsable::<P>() {
            return None;
        }
        Some(unsafe { mem::transmute::<&mut NonNull<()>, &mut &'arn mut P>(&mut self.ptr) })
    }

    pub fn as_ptr(self) -> NonNull<()> {
        self.ptr
    }
}

unsafe impl Sync for ParsedMut<'_, '_> {}

unsafe impl Send for ParsedMut<'_, '_> {}
