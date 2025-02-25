use crate::core::cache::Allocs;
use crate::parsable::parsed::Parsed;
use std::fmt::{Debug, Formatter};
use std::iter;
use std::ptr::null;

#[derive(Default, Copy, Clone)]
pub struct VarMap<'arn, 'grm>(Option<&'arn VarMapNode<'arn, 'grm>>);

impl Debug for VarMap<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Printing varmap:")?;
        for (name, _value) in *self {
            writeln!(f, "- {name}")?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct VarMapNode<'arn, 'grm> {
    next: Option<&'arn Self>,
    key: &'arn str,
    value: Parsed<'arn, 'grm>,
}

pub struct VarMapIterator<'arn, 'grm> {
    current: Option<&'arn VarMapNode<'arn, 'grm>>,
}

impl<'arn, 'grm> Iterator for VarMapIterator<'arn, 'grm> {
    type Item = (&'arn str, Parsed<'arn, 'grm>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => None,
            Some(node) => {
                self.current = node.next;
                Some((node.key, node.value))
            }
        }
    }
}

impl<'arn, 'grm> VarMap<'arn, 'grm> {
    pub fn get(&self, k: &str) -> Option<&'arn Parsed<'arn, 'grm>> {
        let mut node = self.0?;
        loop {
            if node.key == k {
                return Some(&node.value);
            }
            node = node.next?;
        }
    }

    #[must_use]
    pub fn insert(self, key: &'arn str, value: Parsed<'arn, 'grm>, alloc: Allocs<'arn>) -> Self {
        self.extend(iter::once((key, value)), alloc)
    }

    #[must_use]
    pub fn extend<T: IntoIterator<Item = (&'arn str, Parsed<'arn, 'grm>)>>(
        mut self,
        iter: T,
        alloc: Allocs<'arn>,
    ) -> Self {
        for (key, value) in iter {
            self.0 = Some(alloc.alloc(VarMapNode {
                next: self.0,
                key,
                value,
            }))
        }
        self
    }

    pub fn from_iter<T: IntoIterator<Item = (&'arn str, Parsed<'arn, 'grm>)>>(
        iter: T,
        alloc: Allocs<'arn>,
    ) -> Self {
        let s = Self::default();
        s.extend(iter, alloc)
    }

    pub fn as_ptr(&self) -> *const VarMapNode {
        self.0.map(|r| r as *const VarMapNode).unwrap_or(null())
    }
}

impl<'arn, 'grm> IntoIterator for VarMap<'arn, 'grm> {
    type Item = (&'arn str, Parsed<'arn, 'grm>);
    type IntoIter = VarMapIterator<'arn, 'grm>;

    fn into_iter(self) -> Self::IntoIter {
        VarMapIterator { current: self.0 }
    }
}
