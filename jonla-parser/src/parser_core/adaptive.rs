use crate::grammar::{AnnotatedRuleExpr, Block, GrammarFile, Rule};
use crate::parser_core::toposet::TopoSet;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

#[derive(Clone)]
pub struct GrammarState<'grm> {
    rules: HashMap<&'grm str, Arc<RuleState<'grm>>>,
}

impl<'grm> GrammarState<'grm> {
    pub fn new(grammar: &'grm GrammarFile) -> Self {
        let mut s = Self {
            rules: HashMap::new(),
        };
        s.update(grammar).unwrap(); // Cannot fail, since they're all new entries
        s
    }

    pub fn contains_rule(&self, rule: &'grm str) -> bool {
        self.rules.contains_key(rule)
    }

    pub fn get(&self, rule: &'grm str) -> Option<&Vec<BlockState<'grm>>> {
        self.rules.get(rule).map(|r| &r.blocks)
    }

    pub fn update(&mut self, grammar: &'grm GrammarFile) -> Result<(), &'grm str> {
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
struct RuleState<'grm> {
    blocks: Vec<BlockState<'grm>>,
    order: TopoSet<'grm>,
}

impl<'grm> RuleState<'grm> {
    pub fn new(r: &'grm Rule) -> Self {
        let blocks = r.blocks.iter().map(|b| BlockState::new(b)).collect();
        let mut order = TopoSet::new();
        order.update(r);
        Self { blocks, order }
    }

    pub fn update(&mut self, r: &'grm Rule) -> Result<(), ()> {
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
pub struct BlockState<'grm> {
    pub name: &'grm str,
    pub constructors: Vec<&'grm AnnotatedRuleExpr>,
}

impl<'grm> BlockState<'grm> {
    pub fn new(block: &'grm Block) -> Self {
        Self {
            name: &block.0,
            constructors: block.1.iter().collect(),
        }
    }

    pub fn update(&mut self, b: &'grm Block) {
        assert_eq!(self.name, b.0);
        self.constructors.extend(&b.1[..]);
    }
}
