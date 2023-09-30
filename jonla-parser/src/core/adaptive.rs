use crate::core::toposet::TopoSet;
use crate::grammar::grammar::{AnnotatedRuleExpr, Block, GrammarFile, Rule};
use std::collections::HashMap;
use std::{iter, mem};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::core::context::{Raw, RawEnv};

pub struct GrammarState<'b, 'grm> {
    rules: Vec<RuleState<'b, 'grm>>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct RuleId(usize);

impl Display for RuleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'b, 'grm> GrammarState<'b, 'grm> {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    pub fn with(&self, grammar: &'b GrammarFile<'grm>, ctx: &HashMap<&'grm str, Arc<RawEnv<'b, 'grm>>>) -> Result<(Self, impl Iterator<Item=(&'grm str, RuleId)> + 'b), &'grm str> {
        Ok((todo!(), iter::empty()))
    }

    pub fn new_with(grammar: &'b GrammarFile<'grm>) -> (Self, impl Iterator<Item=(&'grm str, RuleId)> + 'b) {
        let rule_iter = grammar.rules.iter().enumerate().map(|(i, rule)| {
            (rule.name, RuleId(i))
        });

        let rules: Arc<HashMap<&'grm str, Arc<RawEnv<'b, 'grm>>>> = Arc::new(rule_iter.clone().map(|(k, v)| (k, Arc::new(RawEnv::from_raw(Raw::Rule(v))))).collect());

        let s: Self = Self {
            rules: grammar.rules.iter().map(|rule| {
                RuleState::new(rule, &rules)
            }).collect(),
        };

        (s, rule_iter)
    }


    pub fn get(&self, rule: RuleId) -> Option<&RuleState<'b, 'grm>> {
        self.rules.get(rule.0).map(|rs| &*rs)
    }
}

#[derive(Clone)]
pub struct RuleState<'b, 'grm> {
    pub name: &'grm str,
    pub blocks: Vec<BlockState<'b, 'grm>>,
    order: TopoSet<'grm>,
    pub args: &'b Vec<&'grm str>,
}

impl<'b, 'grm> RuleState<'b, 'grm> {
    pub fn new(r: &'b Rule<'grm>, ctx: &Arc<HashMap<&'grm str, Arc<RawEnv<'b, 'grm>>>>) -> Self {
        let blocks: Vec<BlockState<'b, 'grm>> =
            r.blocks.iter().map(|b| BlockState::new(b, ctx)).collect();
        let mut order: TopoSet<'grm> = TopoSet::new();
        order.update(r);
        Self {
            name: r.name,
            blocks,
            order,
            args: &r.args,
        }
    }

    pub fn update(&mut self, r: &'b Rule<'grm>, ctx: &Arc<HashMap<&'grm str, Arc<RawEnv<'b, 'grm>>>>) -> Result<(), ()> {
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
                    res[i] = Some(BlockState::new(block, ctx));
                }
                Some(bs) => {
                    bs.update(block, ctx);
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
    pub constructors: Vec<(&'b AnnotatedRuleExpr<'grm>, Arc<HashMap<&'grm str, Arc<RawEnv<'b, 'grm>>>>)>,
}

impl<'b, 'grm> BlockState<'b, 'grm> {
    pub fn new(block: &'b Block<'grm>, ctx: &Arc<HashMap<&'grm str, Arc<RawEnv<'b, 'grm>>>>) -> Self {
        Self {
            name: &block.0,
            constructors: block.1.iter().map(|r| (r, ctx.clone())).collect(),
        }
    }

    pub fn update(&mut self, b: &'b Block<'grm>, ctx: &Arc<HashMap<&'grm str, Arc<RawEnv<'b, 'grm>>>>) {
        assert_eq!(self.name, b.0);
        self.constructors.extend(b.1.iter().map(|r| (r, ctx.clone())));
    }
}
