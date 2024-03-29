#[derive(Default)]
pub struct UnionFind {
    parents: Vec<usize>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct UnionIndex(pub usize);

impl UnionFind {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self) -> UnionIndex {
        let index = self.parents.len();
        self.parents.push(index);
        UnionIndex(index)
    }

    pub fn find(&mut self, index: UnionIndex) -> UnionIndex {
        let mut child = index.0;
        let mut parent = self.parents[child];

        // early exit if root
        if parent == child {
            return UnionIndex(parent);
        }

        let parent_parent = self.parents[parent];

        // early exit if one away from root
        if parent_parent == parent {
            return UnionIndex(parent_parent);
        }

        let mut child_indexes = vec![child, parent];
        child = parent_parent;

        // loop until root is found
        loop {
            parent = self.parents[child];
            if parent == child {
                break;
            }
            child_indexes.push(child);
            child = parent;
        }

        // set parent of each child to root
        for child_index in child_indexes {
            self.parents[child_index] = child
        }

        UnionIndex(parent)
    }

    pub fn union(&mut self, a: UnionIndex, b: UnionIndex) -> UnionIndex {
        let a_root = self.find(a);
        let b_root = self.find(b);
        self.union_left(a_root, b_root);
        a_root
    }

    /// `a` will be the new root.
    /// Constraint: `a` and `b` must be roots.
    pub fn union_left(&mut self, a: UnionIndex, b: UnionIndex) {
        assert_eq!(self.parents[a.0], a.0);
        assert_eq!(self.parents[b.0], b.0);
        self.parents[b.0] = a.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let mut uf = UnionFind::new();
        let x = uf.add();
        let y = uf.add();

        assert_eq!(uf.find(x), x);
        assert_eq!(uf.find(y), y);

        uf.union(x, y);

        assert_eq!(uf.find(x), uf.find(y));
    }
}
