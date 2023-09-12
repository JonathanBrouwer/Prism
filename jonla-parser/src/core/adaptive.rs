use crate::core::toposet::TopoSet;
use crate::grammar::grammar::{Action, AnnotatedRuleExpr, Block, GrammarFile, Rule};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

pub struct GrammarState<'b, 'grm, A: Action<'grm>> {
    rules: HashMap<&'grm str, Arc<RuleState<'b, 'grm, A>>>,
}

impl<'b, 'grm, A: Action<'grm>> GrammarState<'b, 'grm, A> {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn new_with(grammar: &'b GrammarFile<'grm, A>) -> Self {
        let mut s = Self {
            rules: HashMap::new(),
        };
        s.update(grammar).unwrap(); // Cannot fail, since they're all new entries
        s
    }

    pub fn contains_rule(&self, rule: &str) -> bool {
        self.rules.contains_key(rule)
    }

    pub fn get(&self, rule: &str) -> Option<&RuleState<'b, 'grm, A>> {
        self.rules.get(rule).map(|x| &**x)
    }

    pub fn with(&self, grammar: &'b GrammarFile<'grm, A>) -> Result<Self, &'grm str> {
        let mut s = GrammarState {
            rules: self.rules.clone()
        };
        s.update(grammar)?;
        Ok(s)
    }

    fn update(&mut self, grammar: &'b GrammarFile<'grm, A>) -> Result<(), &'grm str> {
        for rule in &grammar.rules {
            match self.rules.entry(&rule.name[..]) {
                Entry::Occupied(mut e) => {
                    let mut clone = (**e.get()).clone();
                    clone.update(rule).map_err(|_| &rule.name[..])?;
                    *e.get_mut() = Arc::new(clone);
                }
                Entry::Vacant(e) => {
                    e.insert(Arc::new(RuleState::new(rule)));
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct RuleState<'b, 'grm, A: Action<'grm>> {
    pub name: &'grm str,
    pub blocks: Vec<BlockState<'b, 'grm, A>>,
    order: TopoSet<'grm>,
    pub args: &'b Vec<&'grm str>,
}

impl<'b, 'grm, A: Action<'grm>> RuleState<'b, 'grm, A> {
    pub fn new(r: &'b Rule<'grm, A>) -> Self {
        let blocks: Vec<BlockState<'b, 'grm, A>> =
            r.blocks.iter().map(|b| BlockState::new(b)).collect();
        let mut order: TopoSet<'grm> = TopoSet::new();
        order.update(r);
        Self {
            name: r.name,
            blocks,
            order,
            args: &r.args,
        }
    }

    pub fn update(&mut self, r: &'b Rule<'grm, A>) -> Result<(), ()> {
        self.order.update(r);

        let order: HashMap<&'grm str, usize> = self
            .order
            .toposort()?
            .into_iter()
            .enumerate()
            .map(|(k, v)| (v, k))
            .collect();

        let mut res = vec![None; order.len()];
        let old_blocks = mem::take(&mut self.blocks);

        for block in old_blocks {
            let i = order[block.name];
            res[i] = Some(block);
        }

        for block in &r.blocks {
            let i = order[&block.0[..]];
            match &mut res[i] {
                None => {
                    res[i] = Some(BlockState::new(block));
                }
                Some(bs) => {
                    bs.update(block);
                }
            }
        }

        self.blocks = res.into_iter().map(|m| m.unwrap()).collect();

        Ok(())
    }
}

#[derive(Clone)]
pub struct BlockState<'b, 'grm, A: Action<'grm>> {
    pub name: &'grm str,
    pub constructors: Vec<&'b AnnotatedRuleExpr<'grm, A>>,
}

impl<'b, 'grm, A: Action<'grm>> BlockState<'b, 'grm, A> {
    pub fn new(block: &'b Block<'grm, A>) -> Self {
        Self {
            name: &block.0,
            constructors: block.1.iter().collect(),
        }
    }

    pub fn update(&mut self, b: &'b Block<'grm, A>) {
        assert_eq!(self.name, b.0);
        self.constructors.extend(&b.1[..]);
    }
}
