use std::fmt::{Debug, Formatter};
use std::iter;
use std::ops::Index;
use std::ptr::null;
use std::sync::Arc;

pub struct List<V>(Option<Arc<GenericEnvNode<V>>>, usize);

impl<V> Clone for List<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

impl<V> Default for List<V> {
    fn default() -> Self {
        Self(None, 0)
    }
}

#[derive(Clone)]
pub struct GenericEnvNode<V> {
    next: Option<Arc<GenericEnvNode<V>>>,
    value: V,
}

impl<V: Debug> Debug for List<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Printing varmap:")?;
        for value in self.iter() {
            writeln!(f, "- {value:?}")?;
        }
        Ok(())
    }
}

pub struct GenericEnvIterator<'a, V> {
    current: Option<&'a GenericEnvNode<V>>,
}

impl<'a, V> Iterator for GenericEnvIterator<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => None,
            Some(node) => {
                self.current = node.next.as_deref();
                Some(&node.value)
            }
        }
    }
}

impl<V> List<V> {
    // pub fn get<NB: ?Sized + Eq>(&self, k: &NB) -> Option<&V> {
    //     let mut node = self.0.as_ref()?;
    //     loop {
    //         if node.name.borrow() == k {
    //             return Some(&node.value);
    //         }
    //         node = node.next.as_ref()?;
    //     }
    // }

    pub fn get_idx(&self, i: usize) -> Option<&V> {
        let mut node = self.0.as_ref()?;
        for _ in 0..i {
            node = node.next.as_ref()?;
        }
        Some(&node.value)
    }

    pub fn iter(&self) -> impl Iterator<Item = &V> {
        GenericEnvIterator {
            current: self.0.as_deref(),
        }
    }
}

impl<V> List<V> {
    #[must_use]
    pub fn cons(&self, value: V) -> Self {
        self.extend(iter::once(value))
    }

    #[must_use]
    pub fn shift(&self, amount: usize) -> Self {
        let mut node = self.0.as_ref();
        for _ in 0..amount {
            node = node.unwrap().next.as_ref();
        }
        Self(node.cloned(), self.1 - amount)
    }
}

impl<V> List<V> {
    #[must_use]
    pub fn insert(&self, value: V) -> Self {
        self.extend(iter::once(value))
    }

    #[must_use]
    pub fn extend<T: IntoIterator<Item = V>>(&self, iter: T) -> Self {
        let mut current: Option<Arc<GenericEnvNode<V>>> = self.0.clone();
        let mut len = self.1;
        for value in iter {
            current = Some(Arc::new(GenericEnvNode {
                next: current,
                value,
            }));
            len += 1;
        }
        Self(current, len)
    }

    pub fn as_ptr(&self) -> *const GenericEnvNode<V> {
        self.0
            .as_ref()
            .map(|r| (&**r) as *const GenericEnvNode<V>)
            .unwrap_or(null())
    }

    pub fn len(&self) -> usize {
        self.1
    }

    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.1, 0);
        self.0.is_none()
    }

    pub fn split(&self) -> Option<(&V, Self)> {
        self.0
            .as_ref()
            .map(|node| (&node.value, Self(node.next.clone(), self.1 - 1)))
    }

    pub fn intersect(&self, other: &Self) -> Self {
        let mut n1 = &self.0;
        let mut n2 = &other.0;

        // Align both pointers
        let mut len = if self.1 > other.1 {
            for _ in 0..(self.1 - other.1) {
                n1 = &n1.as_ref().unwrap().next;
            }
            other.1
        } else {
            for _ in 0..(other.1 - self.1) {
                n2 = &n2.as_ref().unwrap().next;
            }
            self.1
        };

        // Traverse until equal
        while n1.as_ref().map(|p| (&**p) as *const GenericEnvNode<V>)
            != n2.as_ref().map(|p| (&**p) as *const GenericEnvNode<V>)
        {
            n1 = &n1.as_ref().unwrap().next;
            n2 = &n2.as_ref().unwrap().next;
            len -= 1;
        }

        Self(n1.clone(), len)
    }
}

impl<V> FromIterator<V> for List<V> {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let s = Self::default();
        s.extend(iter)
    }
}

impl<V> Index<usize> for List<V> {
    type Output = V;

    fn index(&self, index: usize) -> &Self::Output {
        let mut node = self.0.as_ref().unwrap();
        for _ in 0..index {
            node = node.next.as_ref().unwrap();
        }
        &node.value
    }
}
