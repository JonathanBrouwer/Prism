use std::fmt;
use std::fmt::Formatter;
use std::marker::PhantomData;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;

pub mod leak_slice {
    use super::*;

    pub fn serialize<S: Serializer, T: Serialize>(xs: &[T], s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(xs.len()))?;
        for x in xs {
            seq.serialize_element(x)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>, T: Deserialize<'de>>(deserializer: D) -> Result<&'de [T], D::Error> {
        struct VecVisitor<T> {
            marker: PhantomData<T>,
        }

        impl<'de, T> Visitor<'de> for VecVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = Vec<T>;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::<T>::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }
                Ok(values)
            }
        }

        let visitor = VecVisitor {
            marker: PhantomData,
        };
        let vec: Vec<T> = deserializer.deserialize_seq(visitor)?;
        Ok(vec.leak())
    }
}

pub mod leak {
    use super::*;
    
    pub fn serialize<S: Serializer, T: Serialize>(x: &T, s: S) -> Result<S::Ok, S::Error> {
        x.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>, T: Deserialize<'de>>(deserializer: D) -> Result<&'de T, D::Error> {
        Ok(Box::leak(Box::new(T::deserialize(deserializer)?)))
    }
}