use crate::core::allocs::Allocs;
use crate::parsable::ParseResult;
use std::fmt::{Debug, Formatter};
use std::iter;
use std::ops::Index;
use std::ptr::null;

#[derive(Copy, Clone)]
pub struct GenericEnv<'arn, N: Copy, V: Copy>(Option<&'arn GenericEnvNode<'arn, N, V>>, usize);

impl<'arn, N: Copy + Sized + Sync + Send + 'arn, V: Copy + Sized + Sync + Send + 'arn> ParseResult
    for GenericEnv<'arn, N, V>
{
}

impl<N: Copy, V: Copy> Default for GenericEnv<'_, N, V> {
    fn default() -> Self {
        Self(None, 0)
    }
}

#[derive(Copy, Clone)]
pub struct GenericEnvNode<'arn, N: Copy, V: Copy> {
    next: Option<&'arn GenericEnvNode<'arn, N, V>>,
    name: N,
    value: V,
}

impl<N: Debug + Copy, V: Copy> Debug for GenericEnv<'_, N, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Printing varmap:")?;
        for (name, _value) in *self {
            writeln!(f, "- {name:?}")?;
        }
        Ok(())
    }
}

pub struct GenericEnvIterator<'arn, N: Copy, V: Copy> {
    current: Option<&'arn GenericEnvNode<'arn, N, V>>,
    len_left: usize,
}

impl<N: Copy, V: Copy> Iterator for GenericEnvIterator<'_, N, V> {
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

impl<N: Copy, V: Copy> ExactSizeIterator for GenericEnvIterator<'_, N, V> {}

impl<'arn, N: Copy, V: Copy> IntoIterator for GenericEnv<'arn, N, V> {
    type Item = (N, V);
    type IntoIter = GenericEnvIterator<'arn, N, V>;

    fn into_iter(self) -> Self::IntoIter {
        GenericEnvIterator {
            current: self.0,
            len_left: self.1,
        }
    }
}

impl<N: Copy + Debug + Eq, V: Copy> GenericEnv<'_, N, V> {
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

impl<'arn, V: Copy> GenericEnv<'arn, (), V> {
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

impl<'arn, N: Copy + Debug, V: Copy> GenericEnv<'arn, N, V> {
    #[must_use]
    pub fn insert(self, key: N, value: V, alloc: Allocs<'arn>) -> Self {
        self.extend(iter::once((key, value)), alloc)
    }

    #[must_use]
    pub fn extend<T: IntoIterator<Item = (N, V)>>(mut self, iter: T, alloc: Allocs<'arn>) -> Self {
        for (name, value) in iter {
            self.0 = Some(alloc.alloc(GenericEnvNode {
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

    pub fn as_ptr(&self) -> *const GenericEnvNode<'arn, N, V> {
        self.0
            .map(|r| r as *const GenericEnvNode<'arn, N, V>)
            .unwrap_or(null())
    }

    pub fn len(&self) -> usize {
        self.1
    }

    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.1, 0);
        self.0.is_none()
    }

    pub fn split(&self) -> Option<((N, V), Self)> {
        self.0
            .map(|node| ((node.name, node.value), Self(node.next, self.1 - 1)))
    }

    pub fn intersect(self, other: Self) -> Self {
        let mut n1 = self.0;
        let mut n2 = other.0;

        // Align both pointers
        let mut len = if self.1 > other.1 {
            for _ in 0..(self.1 - other.1) {
                n1 = n1.unwrap().next;
            }
            other.1
        } else {
            for _ in 0..(other.1 - self.1) {
                n2 = n2.unwrap().next;
            }
            self.1
        };

        // Traverse until equal
        while n1.map(|p| p as *const GenericEnvNode<N, V>)
            != n2.map(|p| p as *const GenericEnvNode<N, V>)
        {
            n1 = n1.unwrap().next;
            n2 = n2.unwrap().next;
            len -= 1;
        }

        Self(n1, len)
    }
}

impl<V: Copy> Index<usize> for GenericEnv<'_, (), V> {
    type Output = V;

    fn index(&self, index: usize) -> &Self::Output {
        let mut node = self.0.unwrap();
        for _ in 0..index {
            node = node.next.unwrap();
        }
        &node.value
    }
}
