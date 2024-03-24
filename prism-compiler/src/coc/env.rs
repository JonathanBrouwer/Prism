use crate::union_find::UnionIndex;
use rpds::Vector;
use std::ops::Index;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum EnvEntry {
    // Definitions used during type checking
    CType(UnionIndex),
    CSubst(UnionIndex, UnionIndex),

    // Definitions used during beta reduction
    RType,
    RSubst(UnionIndex, Env),
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct Env(Vector<EnvEntry>);

impl Env {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn cons(&self, e: EnvEntry) -> Self {
        Env(self.0.push_back(e))
    }

    /// Drops the last `count` elements from the Environment
    #[must_use]
    pub fn shift(&self, count: usize) -> Self {
        let mut s = self.0.clone();
        assert!(s.len() >= count);
        for _ in 0..count {
            s.drop_last_mut();
        }
        Env(s)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Index<usize> for Env {
    type Output = EnvEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[self.0.len() - 1 - index]
    }
}
