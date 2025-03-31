use crate::core::input_table::InputTable;
use crate::grammar::identifier::Identifier;
use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::iter;
use std::ops::Index;
use std::ptr::null;
use std::sync::Arc;

pub struct GenericEnv<N, V>(Option<Arc<GenericEnvNode<N, V>>>, usize);

impl<N, V> Clone for GenericEnv<N, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}

impl<N, V> Default for GenericEnv<N, V> {
    fn default() -> Self {
        Self(None, 0)
    }
}

#[derive(Clone)]
pub struct GenericEnvNode<N, V> {
    next: Option<Arc<GenericEnvNode<N, V>>>,
    name: N,
    value: V,
}

impl<N: Debug + Copy, V> Debug for GenericEnv<N, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Printing varmap:")?;
        for (name, _value) in self.iter() {
            writeln!(f, "- {name:?}")?;
        }
        Ok(())
    }
}

pub struct GenericEnvIterator<'a, N, V> {
    current: Option<&'a GenericEnvNode<N, V>>,
}

impl<'a, N, V> Iterator for GenericEnvIterator<'a, N, V> {
    type Item = (&'a N, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => None,
            Some(node) => {
                self.current = node.next.as_deref();
                Some((&node.name, &node.value))
            }
        }
    }
}

impl<V> GenericEnv<Identifier, V> {
    pub fn get_ident(&self, k: Identifier, input: &InputTable) -> Option<&V> {
        let k = k.as_str(input);
        let mut node = self.0.as_ref()?;
        loop {
            if node.name.as_str(input) == k {
                return Some(&node.value);
            }
            node = node.next.as_ref()?;
        }
    }
}

impl<N, V> GenericEnv<N, V> {
    pub fn get<NB: ?Sized + Eq>(&self, k: &NB) -> Option<&V>
    where
        N: Borrow<NB>,
    {
        let mut node = self.0.as_ref()?;
        loop {
            if node.name.borrow() == k {
                return Some(&node.value);
            }
            node = node.next.as_ref()?;
        }
    }

    pub fn get_idx(&self, i: usize) -> Option<&V> {
        let mut node = self.0.as_ref()?;
        for _ in 0..i {
            node = node.next.as_ref()?;
        }
        Some(&node.value)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&N, &V)> {
        GenericEnvIterator {
            current: self.0.as_deref(),
        }
    }

    pub fn iter_cloned(&self) -> impl Iterator<Item = (N, V)>
    where
        N: Clone,
        V: Clone,
    {
        self.iter().map(|(k, v)| (k.clone(), v.clone()))
    }
}

impl<V> GenericEnv<(), V> {
    #[must_use]
    pub fn cons(&self, value: V) -> Self {
        self.extend(iter::once(((), value)))
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

impl<N: Debug, V> GenericEnv<N, V> {
    #[must_use]
    pub fn insert(&self, key: N, value: V) -> Self {
        self.extend(iter::once((key, value)))
    }

    #[must_use]
    pub fn extend<T: IntoIterator<Item = (N, V)>>(&self, iter: T) -> Self {
        let mut current: Option<Arc<GenericEnvNode<N, V>>> = self.0.clone();
        let mut len = self.1;
        for (name, value) in iter {
            current = Some(Arc::new(GenericEnvNode {
                next: current,
                name,
                value,
            }));
            len += 1;
        }
        Self(current, len)
    }

    pub fn as_ptr(&self) -> *const GenericEnvNode<N, V> {
        self.0
            .as_ref()
            .map(|r| (&**r) as *const GenericEnvNode<N, V>)
            .unwrap_or(null())
    }

    pub fn len(&self) -> usize {
        self.1
    }

    pub fn is_empty(&self) -> bool {
        debug_assert_eq!(self.1, 0);
        self.0.is_none()
    }

    pub fn split(&self) -> Option<((&N, &V), Self)> {
        self.0.as_ref().map(|node| {
            (
                (&node.name, &node.value),
                Self(node.next.clone(), self.1 - 1),
            )
        })
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
        while n1.as_ref().map(|p| (&**p) as *const GenericEnvNode<N, V>)
            != n2.as_ref().map(|p| (&**p) as *const GenericEnvNode<N, V>)
        {
            n1 = &n1.as_ref().unwrap().next;
            n2 = &n2.as_ref().unwrap().next;
            len -= 1;
        }

        Self(n1.clone(), len)
    }
}

impl<N: Debug, V> FromIterator<(N, V)> for GenericEnv<N, V> {
    fn from_iter<T: IntoIterator<Item = (N, V)>>(iter: T) -> Self {
        let s = Self::default();
        s.extend(iter)
    }
}

impl<V> Index<usize> for GenericEnv<(), V> {
    type Output = V;

    fn index(&self, index: usize) -> &Self::Output {
        let mut node = self.0.as_ref().unwrap();
        for _ in 0..index {
            node = node.next.as_ref().unwrap();
        }
        &node.value
    }
}
