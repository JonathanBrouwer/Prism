use crate::grammar::Rule;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

#[derive(Clone)]
pub struct TopoSet<'grm> {
    map: HashMap<&'grm str, Arc<HashSet<&'grm str>>>,
}

impl<'grm> TopoSet<'grm> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn update(&mut self, grm: &Rule<'grm>) {
        for b in grm.blocks.windows(2) {
            let b1 = &b[0].0[..];
            let b2 = &b[1].0[..];

            match self.map.entry(b1) {
                Entry::Occupied(mut e) => {
                    if !e.get().contains(b2) {
                        let mut set = (**e.get()).clone();
                        set.insert(b2);
                        *e.get_mut() = Arc::new(set);
                    }
                }
                Entry::Vacant(e) => {
                    e.insert(Arc::new(HashSet::from([b2])));
                }
            }
        }
        // Insert final entry
        if let Entry::Vacant(e) = self.map.entry(&grm.blocks.last().unwrap().0) {
            e.insert(Arc::new(HashSet::new()));
        }
    }

    pub fn toposort(&self) -> Result<Vec<&'grm str>, ()> {
        let mut result = Vec::new();
        let mut counts: HashMap<&'grm str, usize> = HashMap::new();

        for (k, nbs) in &self.map {
            counts.entry(k).or_insert(0);
            for nb in &**nbs {
                *counts.entry(nb).or_insert(0) += 1;
            }
        }

        let mut todo: VecDeque<&'grm str> = counts
            .iter()
            .filter(|(_, v)| **v == 0)
            .map(|(k, _)| *k)
            .collect();

        while let Some(k) = todo.pop_front() {
            counts.remove(k);
            result.push(k);

            for nb in self.map[k].iter() {
                let count = counts.get_mut(nb).unwrap();
                *count -= 1;
                if *count == 0 {
                    todo.push_back(nb);
                }
            }
        }

        if counts.len() == 0 {
            Ok(result)
        } else {
            Err(())
        }
    }
}