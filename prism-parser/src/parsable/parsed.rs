use std::any::{Any, type_name};
use std::fmt::{Debug, Formatter};
use std::hash::{DefaultHasher, Hasher};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::Arc;

#[derive(Clone)]
pub struct Parsed {
    value: Arc<dyn Any + Send + Sync>,
    pub(crate) name: &'static str,
}

impl Debug for Parsed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parsed(ANONYMOUS PARSED OBJECT)")
    }
}

impl Parsed {
    pub fn into_value<P: Any + Send + Sync>(self) -> Arc<P> {
        let name = self.name;
        self.try_into_value().unwrap_or_else(|| {
            panic!(
                "Expected wrong king of Parsable. Expected {}, got {}",
                type_name::<P>(),
                name
            )
        })
    }

    pub fn value_ref<P: Any + Send + Sync>(&self) -> &P {
        let name = self.name;
        self.try_value_ref::<P>().unwrap_or_else(|| {
            panic!(
                "Expected wrong king of Parsable. Expected {}, got {}",
                type_name::<P>(),
                name
            )
        })
    }

    pub fn value_cloned<P: Any + Send + Sync>(&self) -> Arc<P> {
        let name = self.name;
        self.try_value_cloned().unwrap_or_else(|| {
            panic!(
                "Expected wrong king of Parsable. Expected {}, got {}",
                type_name::<P>(),
                name
            )
        })
    }

    pub fn try_into_value<P: Any + Send + Sync>(self) -> Option<Arc<P>> {
        Arc::downcast(self.value).ok()
    }

    pub fn try_value_ref<P: Any + Send + Sync>(&self) -> Option<&P> {
        let v: &dyn Any = self.value.as_ref();
        v.downcast_ref()
    }

    pub fn try_value_cloned<P: Any + Send + Sync>(&self) -> Option<Arc<P>> {
        self.clone().try_into_value()
    }

    pub fn as_ptr(&self) -> NonNull<()> {
        NonNull::from(self.value.as_ref()).cast()
    }
}

pub trait ArcExt: Sized {
    fn to_parsed(self) -> Parsed;
}

impl<P: Any + Send + Sync> ArcExt for Arc<P> {
    fn to_parsed(self) -> Parsed {
        Parsed {
            value: self,
            name: type_name::<P>(),
        }
    }
}

unsafe impl Sync for Parsed {}

unsafe impl Send for Parsed {}
