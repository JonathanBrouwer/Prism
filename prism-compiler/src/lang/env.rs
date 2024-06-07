use crate::lang::{TcEnv};
use rpds::Vector;
use crate::lang::UnionIndex;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct UniqueVariableId(usize);

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum EnvEntry {
    // Definitions used during type checking
    CType(UniqueVariableId, UnionIndex),
    CSubst(UnionIndex, UnionIndex),

    // Definitions used during beta reduction
    RType(UniqueVariableId),
    RSubst(UnionIndex, Env),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct GenericEnv<T>(Vector<T>);

impl<T> Default for GenericEnv<T> {
    fn default() -> Self {
        Self(Vector::default())
    }
}

pub type Env = GenericEnv<EnvEntry>;

impl<T> GenericEnv<T> {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn cons(&self, e: T) -> Self {
        Self(self.0.push_back(e))
    }

    /// Drops the last `count` elements from the Environment
    #[must_use]
    pub fn shift(&self, count: usize) -> Self {
        let mut s = self.0.clone();
        assert!(s.len() >= count);
        for _ in 0..count {
            s.drop_last_mut();
        }
        Self(s)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            None
        } else {
            self.0.get(self.0.len() - 1 - index)
        }
    }
}

impl<T> std::ops::Index<usize> for GenericEnv<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl TcEnv {
    pub fn new_tc_id(&mut self) -> UniqueVariableId {
        let id = UniqueVariableId(self.tc_id);
        self.tc_id += 1;
        id
    }
}
