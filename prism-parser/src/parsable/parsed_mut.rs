use crate::parsable::ParseResult;
use crate::parsable::parsed::checksum_parsable;
use std::any::type_name;
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;

pub struct ParsedMut<'arn> {
    ptr: NonNull<()>,
    checksum: u64,
    pub(crate) name: &'static str,
    phantom_data: PhantomData<(&'arn (), &'arn ())>,
}

impl<'arn> ParsedMut<'arn> {
    pub fn from_value<P: ParseResult<'arn>>(p: &'arn mut P) -> Self {
        ParsedMut {
            ptr: NonNull::from(p).cast(),
            checksum: checksum_parsable::<P>(),
            name: type_name::<P>(),
            phantom_data: Default::default(),
        }
    }

    pub fn into_value<P: ParseResult<'arn>>(self) -> &'arn mut P {
        let name = self.name;
        self.try_into_value().unwrap_or_else(|| {
            panic!(
                "Expected wrong king of Parsable. Expected {}, got {}",
                type_name::<P>(),
                name
            )
        })
    }

    pub fn into_value_mut<P: ParseResult<'arn>>(&mut self) -> &mut &'arn mut P {
        let name = self.name;
        self.try_into_value_mut().unwrap_or_else(|| {
            panic!(
                "Expected wrong king of Parsable. Expected {}, got {}",
                type_name::<P>(),
                name
            )
        })
    }

    pub fn try_into_value<P: ParseResult<'arn>>(self) -> Option<&'arn mut P> {
        if self.checksum != checksum_parsable::<P>() {
            return None;
        }
        Some(unsafe { self.ptr.cast::<P>().as_mut() })
    }

    pub fn try_into_value_mut<P: ParseResult<'arn>>(&mut self) -> Option<&mut &'arn mut P> {
        if self.checksum != checksum_parsable::<P>() {
            return None;
        }
        Some(unsafe { mem::transmute::<&mut NonNull<()>, &mut &'arn mut P>(&mut self.ptr) })
    }

    pub fn as_ptr(self) -> NonNull<()> {
        self.ptr
    }
}

unsafe impl Sync for ParsedMut<'_> {}

unsafe impl Send for ParsedMut<'_> {}
