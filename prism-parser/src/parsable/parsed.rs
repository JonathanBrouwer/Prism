use crate::parsable::Parsable;
use std::any::type_name;
use std::fmt::{Debug, Formatter};
use std::hash::{DefaultHasher, Hasher};
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Copy, Clone)]
pub struct Parsed<'arn, 'grm> {
    ptr: NonNull<()>,
    checksum: u64,
    name: &'static str,
    phantom_data: PhantomData<(&'arn (), &'grm ())>,
}

impl<'arn, 'grm: 'arn> Debug for Parsed<'arn, 'grm> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parsed(ANONYMOUS PARSED OBJECT)")
    }
}

impl<'arn, 'grm: 'arn> Parsed<'arn, 'grm> {
    pub fn from_value<P: Parsable<'arn, 'grm>>(p: &'arn P) -> Self {
        Parsed {
            ptr: NonNull::from(p).cast(),
            checksum: checksum_parsable::<P>(),
            name: type_name::<P>(),
            phantom_data: Default::default(),
        }
    }

    pub fn into_value<P: Parsable<'arn, 'grm>>(self) -> &'arn P {
        self.try_into_value().expect(&format!(
            "Expected wrong king of Parsable. Expected {}, got {}",
            type_name::<P>(),
            self.name
        ))
    }

    pub fn try_into_value<P: Parsable<'arn, 'grm>>(self) -> Option<&'arn P> {
        if self.checksum != checksum_parsable::<P>() {
            return None;
        }
        Some(unsafe { self.ptr.cast::<P>().as_ref() })
    }

    pub fn as_ptr(self) -> NonNull<()> {
        self.ptr
    }
}

fn checksum_parsable<'arn, 'grm: 'arn, P: Parsable<'arn, 'grm> + 'arn>() -> u64 {
    let mut hash = DefaultHasher::new();

    hash.write(type_name::<P>().as_bytes());

    hash.finish()
}

unsafe impl Sync for Parsed<'_, '_> {}

unsafe impl Send for Parsed<'_, '_> {}
