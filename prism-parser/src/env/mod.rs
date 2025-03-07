use crate::core::allocs::Allocs;
use std::fmt::{Debug, Formatter};
use std::iter;
use std::ptr::null;

#[derive(Copy, Clone)]
pub struct GenericerEnv<'arn, N: Eq + Copy, V: Copy>(Option<&'arn GenericerEnvNode<'arn, N, V>>);

impl<'arn, N: Debug + Eq + Copy, V: Copy> Default for GenericerEnv<'arn, N, V> {
    fn default() -> Self {
        Self(None)
    }
}

#[derive(Copy, Clone)]
pub struct GenericerEnvNode<'arn, N: Eq + Copy, V: Copy> {
    name: N,
    value: V,
    next: Option<&'arn GenericerEnvNode<'arn, N, V>>,
}

impl<'arn, N: Debug + Eq + Copy, V: Copy> Debug for GenericerEnv<'arn, N, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Printing varmap:")?;
        for (name, _value) in *self {
            writeln!(f, "- {name:?}")?;
        }
        Ok(())
    }
}

pub struct GenericerEnvIterator<'arn, N: Eq + Copy, V: Copy> {
    current: Option<&'arn GenericerEnvNode<'arn, N, V>>,
}

impl<'arn, N: Eq + Copy, V: Copy> Iterator for GenericerEnvIterator<'arn, N, V> {
    type Item = (N, V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => None,
            Some(node) => {
                self.current = node.next;
                Some((node.name, node.value))
            }
        }
    }
}

impl<'arn, N: Eq + Copy, V: Copy> IntoIterator for GenericerEnv<'arn, N, V> {
    type Item = (N, V);
    type IntoIter = GenericerEnvIterator<'arn, N, V>;

    fn into_iter(self) -> Self::IntoIter {
        GenericerEnvIterator { current: self.0 }
    }
}

impl<'arn, N: Eq + Copy + Debug, V: Copy> GenericerEnv<'arn, N, V> {
    pub fn get(&self, k: N) -> Option<&'arn V> {
        let mut node = self.0?;
        loop {
            if node.name == k {
                return Some(&node.value);
            }
            node = node.next?;
        }
    }

    #[must_use]
    pub fn cons(self, key: N, value: V, alloc: Allocs<'arn>) -> Self {
        self.extend(iter::once((key, value)), alloc)
    }

    #[must_use]
    pub fn extend<T: IntoIterator<Item = (N, V)>>(mut self, iter: T, alloc: Allocs<'arn>) -> Self {
        for (name, value) in iter {
            self.0 = Some(alloc.alloc(GenericerEnvNode {
                next: self.0,
                name,
                value,
            }))
        }
        self
    }

    pub fn from_iter<T: IntoIterator<Item = (N, V)>>(iter: T, alloc: Allocs<'arn>) -> Self {
        let s = Self::default();
        s.extend(iter, alloc)
    }

    pub fn as_ptr(&self) -> *const GenericerEnvNode<'arn, N, V> {
        self.0
            .map(|r| r as *const GenericerEnvNode<'arn, N, V>)
            .unwrap_or(null())
    }
}
