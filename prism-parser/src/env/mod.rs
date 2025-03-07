use crate::core::allocs::Allocs;
use crate::parsable::ParseResult;
use std::fmt::{Debug, Formatter};
use std::iter;
use std::ops::Index;
use std::ptr::null;

#[derive(Copy, Clone)]
pub struct GenericerEnv<'arn, N: Copy, V: Copy>(Option<&'arn GenericerEnvNode<'arn, N, V>>, usize);

impl<'arn, 'grm: 'arn, N: Copy + Sized + Sync + Send + 'arn, V: Copy + Sized + Sync + Send + 'arn>
    ParseResult<'arn, 'grm> for GenericerEnv<'arn, N, V>
{
}

impl<N: Debug + Copy, V: Copy> Default for GenericerEnv<'_, N, V> {
    fn default() -> Self {
        Self(None, 0)
    }
}

#[derive(Copy, Clone)]
pub struct GenericerEnvNode<'arn, N: Copy, V: Copy> {
    name: N,
    value: V,
    next: Option<&'arn GenericerEnvNode<'arn, N, V>>,
}

impl<N: Debug + Copy, V: Copy> Debug for GenericerEnv<'_, N, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Printing varmap:")?;
        for (name, _value) in *self {
            writeln!(f, "- {name:?}")?;
        }
        Ok(())
    }
}

pub struct GenericerEnvIterator<'arn, N: Copy, V: Copy> {
    current: Option<&'arn GenericerEnvNode<'arn, N, V>>,
    len_left: usize,
}

impl<N: Copy, V: Copy> Iterator for GenericerEnvIterator<'_, N, V> {
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len_left, Some(self.len_left))
    }
}

impl<N: Copy, V: Copy> ExactSizeIterator for GenericerEnvIterator<'_, N, V> {}

impl<'arn, N: Copy, V: Copy> IntoIterator for GenericerEnv<'arn, N, V> {
    type Item = (N, V);
    type IntoIter = GenericerEnvIterator<'arn, N, V>;

    fn into_iter(self) -> Self::IntoIter {
        GenericerEnvIterator {
            current: self.0,
            len_left: self.1,
        }
    }
}

impl<N: Copy + Debug + Eq, V: Copy> GenericerEnv<'_, N, V> {
    pub fn get(&self, k: N) -> Option<V> {
        let mut node = self.0?;
        loop {
            if node.name == k {
                return Some(node.value);
            }
            node = node.next?;
        }
    }

    pub fn get_idx(&self, i: usize) -> Option<V> {
        let mut node = self.0?;
        for _ in 0..i {
            node = node.next?;
        }
        Some(node.value)
    }
}

impl<'arn, V: Copy> GenericerEnv<'arn, (), V> {
    #[must_use]
    pub fn cons(self, value: V, alloc: Allocs<'arn>) -> Self {
        self.extend(iter::once(((), value)), alloc)
    }

    #[must_use]
    pub fn shift(self, amount: usize) -> Self {
        let mut node = self.0;
        for _ in 0..amount {
            node = node.unwrap().next;
        }
        Self(node, self.1 - amount)
    }
}

impl<'arn, N: Copy + Debug, V: Copy> GenericerEnv<'arn, N, V> {
    #[must_use]
    pub fn insert(self, key: N, value: V, alloc: Allocs<'arn>) -> Self {
        self.extend(iter::once((key, value)), alloc)
    }

    #[must_use]
    pub fn extend<T: IntoIterator<Item = (N, V)>>(mut self, iter: T, alloc: Allocs<'arn>) -> Self {
        for (name, value) in iter {
            self.0 = Some(alloc.alloc(GenericerEnvNode {
                next: self.0,
                name,
                value,
            }));
            self.1 += 1;
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

    pub fn len(&self) -> usize {
        self.1
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    pub fn split(&self) -> Option<((N, V), Self)> {
        self.0
            .map(|node| ((node.name, node.value), Self(node.next, self.1 - 1)))
    }
}

impl<V: Copy> Index<usize> for GenericerEnv<'_, (), V> {
    type Output = V;

    fn index(&self, index: usize) -> &Self::Output {
        let mut node = self.0.unwrap();
        for _ in 0..index {
            node = node.next.unwrap();
        }
        &node.value
    }
}
