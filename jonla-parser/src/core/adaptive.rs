use crate::core::toposet::TopoSet;
use crate::grammar::grammar::{AnnotatedRuleExpr, Block, GrammarFile, Rule};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

#[derive(Clone)]
pub struct GrammarState<'b, 'grm> {
    rules: HashMap<&'grm str, Arc<RuleState<'b, 'grm>>>,
}

impl<'b, 'grm> GrammarState<'b, 'grm> {
    pub fn new(grammar: &'b GrammarFile<'grm>) -> Self {
        let mut s = Self {
            rules: HashMap::new(),
        };
        s.update(grammar).unwrap(); // Cannot fail, since they're all new entries
        s
    }

    pub fn contains_rule(&self, rule: &'grm str) -> bool {
        self.rules.contains_key(rule)
    }

    pub fn get(&self, rule: &'grm str) -> Option<&Vec<BlockState<'b, 'grm>>> {
        self.rules.get(rule).map(|r| &r.blocks)
    }

    pub fn update(&mut self, grammar: &'b GrammarFile<'grm>) -> Result<(), &'grm str> {
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
struct RuleState<'b, 'grm> {
    blocks: Vec<BlockState<'b, 'grm>>,
    order: TopoSet<'grm>,
}

impl<'b, 'grm> RuleState<'b, 'grm> {
    pub fn new(r: &'b Rule<'grm>) -> Self {
        let blocks: Vec<BlockState<'b, 'grm>> =
            r.blocks.iter().map(|b| BlockState::new(b)).collect();
        let mut order: TopoSet<'grm> = TopoSet::new();
        order.update(r);
        Self { blocks, order }
    }

    pub fn update(&mut self, r: &'b Rule<'grm>) -> Result<(), ()> {
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
pub struct BlockState<'b, 'grm> {
    pub name: &'grm str,
    pub constructors: Vec<&'b AnnotatedRuleExpr<'grm>>,
}

impl<'b, 'grm> BlockState<'b, 'grm> {
    pub fn new(block: &'b Block<'grm>) -> Self {
        Self {
            name: &block.0,
            constructors: block.1.iter().collect(),
        }
    }

    pub fn update(&mut self, b: &'b Block<'grm>) {
        assert_eq!(self.name, b.0);
        self.constructors.extend(&b.1[..]);
    }
}
