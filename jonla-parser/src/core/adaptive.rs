use crate::core::toposet::TopoSet;
use crate::grammar::grammar::{Action, AnnotatedRuleExpr, Block, GrammarFile, Rule};
use std::collections::HashMap;
use std::{iter, mem};
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

pub struct GrammarState<'b, 'grm, A: Action<'grm>> {
    rules: Vec<RuleState<'b, 'grm, A>>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct RuleId(usize);

impl Display for RuleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'b, 'grm, A: Action<'grm>> GrammarState<'b, 'grm, A> {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    pub fn new_with(grammar: &'b GrammarFile<'grm, A>) -> (Self, impl Iterator<Item=(&'grm str, RuleId)>) {
        let mut s = Self {
            rules: Vec::new(),
        };
        let i = s.update(grammar).unwrap(); // Cannot fail, since they're all new entries
        (s, i)
    }

    pub fn get(&self, rule: RuleId) -> Option<&RuleState<'b, 'grm, A>> {
        self.rules.get(rule.0).map(|x| &*x)
    }

    pub fn with(&self, grammar: &'b GrammarFile<'grm, A>) -> Result<(Self, impl Iterator<Item=(&'grm str, RuleId)>), &'grm str> {
        let mut s = GrammarState {
            rules: self.rules.clone()
        };
        let iter = s.update(grammar)?;
        Ok((s, iter))
    }

    fn update(&mut self, grammar: &'b GrammarFile<'grm, A>) -> Result<impl Iterator<Item=(&'grm str, RuleId)>, &'grm str> {
        // for rule in &grammar.rules {
        //     match self.rules.entry(&rule.name[..]) {
        //         Entry::Occupied(mut e) => {
        //             let mut clone = (**e.get()).clone();
        //             clone.update(rule).map_err(|_| &rule.name[..])?;
        //             *e.get_mut() = Arc::new(clone);
        //         }
        //         Entry::Vacant(e) => {
        //             e.insert(Arc::new(RuleState::new(rule)));
        //         }
        //     }
        // }
        // Ok(())
        todo!();
        Ok(iter::empty())
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
