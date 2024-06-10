use crate::core::pos::Pos;
use crate::core::toposet::TopoSet;
use crate::grammar::{AnnotatedRuleExpr, Block, GrammarFile, Rule};
use crate::rule_action::RuleAction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::{iter, mem};

pub struct GrammarState<'arn, 'grm> {
    rules: Vec<Arc<RuleState<'arn, 'grm>>>,
    last_mut_pos: Option<Pos>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct RuleId(usize);

impl Display for RuleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'arn, 'grm> Default for GrammarState<'arn, 'grm> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum AdaptError<'grm> {
    InvalidRuleMutation(&'grm str),
    SamePos(Pos),
}

impl<'arn, 'grm: 'arn> GrammarState<'arn, 'grm> {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            last_mut_pos: None,
        }
    }

    pub fn with(
        &self,
        grammar: &'arn GrammarFile<'grm, RuleAction<'arn, 'grm>>,
        ctx: impl Iterator<Item = (&'grm str, RuleId)>,
        pos: Option<Pos>,
    ) -> Result<(Self, impl Iterator<Item = (&'grm str, RuleId)> + 'arn), AdaptError<'grm>> {
        let mut s = Self {
            rules: self.rules.clone(),
            last_mut_pos: pos,
        };

        if let Some(pos) = pos {
            if let Some(last_mut_pos) = self.last_mut_pos {
                if pos == last_mut_pos {
                    return Err(AdaptError::SamePos(pos));
                }
            }
        }

        let mut ctx: HashMap<_, _> = ctx.collect();

        let mut result = vec![];
        for new_rule in &grammar.rules {
            let rule = if let Some(rule) = ctx.get(new_rule.name) {
                *rule
            } else {
                s.rules.push(Arc::new(RuleState::new_empty(
                    new_rule.name,
                    &new_rule.args,
                )));
                RuleId(s.rules.len() - 1)
            };
            result.push((new_rule.name, rule));
            ctx.insert(new_rule.name, rule);
        }

        let ctx = Arc::new(ctx);

        for (&(_, id), rule) in result.iter().zip(grammar.rules.iter()) {
            let mut r = (*s.rules[id.0]).clone();
            r.update(rule, &ctx)
                .map_err(|_| AdaptError::InvalidRuleMutation(rule.name))?;
            s.rules[id.0] = Arc::new(r);
        }

        Ok((s, result.into_iter()))
    }

    pub fn new_with(
        grammar: &'arn GrammarFile<'grm, RuleAction<'arn, 'grm>>,
    ) -> (Self, impl Iterator<Item = (&'grm str, RuleId)> + 'arn) {
        GrammarState::new()
            .with(grammar, iter::empty(), None)
            .unwrap()
    }

    pub fn get(&self, rule: RuleId) -> Option<&RuleState<'arn, 'grm>> {
        self.rules.get(rule.0).map(|v| &**v)
    }

    pub fn unique_id(&self) -> GrammarStateId {
        GrammarStateId(self.rules.as_ptr() as usize)
    }
}

// TODO instead of one global GrammarStateId, we can track this per rule. Create a graph of rule ids and update the id when one of its components changes
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct GrammarStateId(usize);

#[derive(Clone)]
pub struct RuleState<'arn, 'grm> {
    pub name: &'grm str,
    pub blocks: Vec<BlockState<'arn, 'grm>>,
    order: TopoSet<'grm>,
    pub arg_names: &'arn Vec<&'grm str>,
}

pub enum UpdateError {
    ToposortCycle,
}

impl<'arn, 'grm> RuleState<'arn, 'grm> {
    pub fn new_empty(name: &'grm str, arg_names: &'arn Vec<&'grm str>) -> Self {
        Self {
            name,
            blocks: Vec::new(),
            order: TopoSet::new(),
            arg_names,
        }
    }

    pub fn update(
        &mut self,
        r: &'arn Rule<'grm, RuleAction<'arn, 'grm>>,
        ctx: &Arc<HashMap<&'grm str, RuleId>>,
    ) -> Result<(), UpdateError> {
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
            let i = order[block.0];
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
pub struct BlockState<'arn, 'grm> {
    pub name: &'grm str,
    pub constructors: Vec<Constructor<'arn, 'grm>>,
}

pub type Constructor<'arn, 'grm> = (
    &'arn AnnotatedRuleExpr<'grm, RuleAction<'arn, 'grm>>,
    Arc<HashMap<&'grm str, RuleId>>,
);

impl<'arn, 'grm> BlockState<'arn, 'grm> {
    pub fn new(
        block: &'arn Block<'grm, RuleAction<'arn, 'grm>>,
        ctx: &Arc<HashMap<&'grm str, RuleId>>,
    ) -> Self {
        Self {
            name: block.0,
            constructors: block.1.iter().map(|r| (r, ctx.clone())).collect(),
        }
    }

    pub fn update(
        &mut self,
        b: &'arn Block<'grm, RuleAction<'arn, 'grm>>,
        ctx: &Arc<HashMap<&'grm str, RuleId>>,
    ) {
        assert_eq!(self.name, b.0);
        self.constructors
            .extend(b.1.iter().map(|r| (r, ctx.clone())));
    }
}
