use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

#[derive(Clone, Serialize, Eq, PartialEq, Hash)]
pub enum Cow<'a, T: 'a> {
    Borrowed(&'a T),
    Owned(T),
}

impl<T> AsRef<T> for Cow<'_, T> {
    fn as_ref(&self) -> &T {
        match &self {
            Cow::Borrowed(v) => v,
            Cow::Owned(v) => v,
        }
    }
}

impl<T> Deref for Cow<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        match *self {
            Cow::Borrowed(borrowed) => borrowed,
            Cow::Owned(ref owned) => owned,
        }
    }
}

impl<T: Debug> Debug for Cow<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Cow::Borrowed(v) => v.fmt(f),
            Cow::Owned(v) => v.fmt(f),
        }
    }
}

impl<'de, 'a, T> Deserialize<'de> for Cow<'a, T>
where
    T: Deserialize<'de>,
{
    #[inline]
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(Cow::Owned)
    }
}
