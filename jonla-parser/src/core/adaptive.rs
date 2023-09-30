use crate::core::toposet::TopoSet;
use crate::grammar::grammar::{Action, AnnotatedRuleExpr, Block, GrammarFile, Rule};
use std::collections::HashMap;
use std::{mem};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

pub struct GrammarState<'b, 'grm, A: Action<'grm>> {
    rules: Vec<RuleState<'b, 'grm, A>>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct RuleId<'grm, A>(usize, PhantomData<A>, PhantomData<&'grm ()>);

impl<'grm, A> Clone for RuleId<'grm, A> {
    fn clone(&self) -> Self {
        RuleId(self.0, Default::default(), Default::default())
    }
}
impl<'grm, A> Copy for RuleId<'grm, A> {

}


impl<'grm, A: Action<'grm>> Display for RuleId<'grm, A> {
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

    pub fn new_with(grammar: &'b GrammarFile<'grm, A>) -> (Self, impl Iterator<Item=(&'grm str, RuleId<'grm, A>)> + 'b) {
        let rules: Arc<HashMap<&'grm str, RuleId<'grm, A>>> = Arc::new(grammar.rules.iter().enumerate().map(|(i, rule)| {
            (rule.name, RuleId(i, Default::default(), Default::default()))
        }).collect());

        let s: Self = Self {
            rules: grammar.rules.iter().map(|rule| RuleState::new(rule, &rules)).collect(),
        };

        (s, grammar.rules.iter().enumerate().map(|(i, rule)| {
            (rule.name, RuleId(i, Default::default(), Default::default()))
        }))
    }

    pub fn get(&self, rule: RuleId<'grm, A>) -> Option<&RuleState<'b, 'grm, A>> {
        self.rules.get(rule.0).map(|x| &*x)
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
    pub fn new(r: &'b Rule<'grm, A>, ctx: &Arc<HashMap<&'grm str, RuleId<'grm, A>>>) -> Self {
        let blocks: Vec<BlockState<'b, 'grm, A>> =
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

    pub fn update(&mut self, r: &'b Rule<'grm, A>, ctx: &Arc<HashMap<&'grm str, RuleId<'grm, A>>>) -> Result<(), ()> {
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
pub struct BlockState<'b, 'grm, A: Action<'grm>> {
    pub name: &'grm str,
    pub constructors: Vec<(&'b AnnotatedRuleExpr<'grm, A>, Arc<HashMap<&'grm str, RuleId<'grm, A>>>)>,
}

impl<'b, 'grm, A: Action<'grm>> BlockState<'b, 'grm, A> {
    pub fn new(block: &'b Block<'grm, A>, ctx: &Arc<HashMap<&'grm str, RuleId<'grm, A>>>) -> Self {
        Self {
            name: &block.0,
            constructors: block.1.iter().map(|r| (r, ctx.clone())).collect(),
        }
    }

    pub fn update(&mut self, b: &'b Block<'grm, A>, ctx: &Arc<HashMap<&'grm str, RuleId<'grm, A>>>) {
        assert_eq!(self.name, b.0);
        self.constructors.extend(b.1.iter().map(|r| (r, ctx.clone())));
    }
}
