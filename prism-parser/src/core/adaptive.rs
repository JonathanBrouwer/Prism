use crate::core::cache::Allocs;
use crate::core::pos::Pos;
use crate::grammar::{AnnotatedRuleExpr, Block, GrammarFile, Rule};
use crate::parser::var_map::{VarMap, VarMapValue};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::{iter, mem};
use std::alloc::alloc;

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

    /// Adapt this grammar state with `grammar`.
    /// To unify rules, use the names of the rules in `ctx`
    pub fn adapt_with(
        &self,
        grammar: &'arn GrammarFile<'arn, 'grm>,
        ctx: VarMap<'arn, 'grm>,
        pos: Option<Pos>,
        alloc: Allocs<'arn>,
    ) -> Result<(Self, VarMap<'arn, 'grm>), AdaptError<'grm>> {
        // Create a clone of self as a starting point
        let mut s = Self {
            rules: self.rules.clone(),
            last_mut_pos: pos,
        };

        // If we already tried to adapt at this position before, crash to prevent infinite loop
        if let Some(pos) = pos {
            if let Some(last_mut_pos) = self.last_mut_pos {
                if pos == last_mut_pos {
                    return Err(AdaptError::SamePos(pos));
                }
            }
        }

        // Create a new ruleid or find an existing rule id for each rule that is adopted
        let mut new_ctx = ctx;
        let result: Vec<_> = grammar
            .rules
            .iter()
            .map(|new_rule| {
                let rule = if let Some(rule) = ctx.get(new_rule.name) {
                    rule.as_rule_id().expect("Can only adapt rule id")
                } else {
                    s.rules.push(Arc::new(RuleState::new_empty(
                        new_rule.name,
                        &new_rule.args,
                    )));
                    RuleId(s.rules.len() - 1)
                };
                new_ctx = new_ctx.extend(
                    iter::once((new_rule.name, VarMapValue::new_rule(rule, alloc))),
                    alloc,
                );
                (new_rule.name, rule)
            })
            .collect();

        // Update each rule that is to be adopted, stored in `result`
        for (&(_, id), rule) in result.iter().zip(grammar.rules.iter()) {
            s.rules[id.0] = Arc::new(s.rules[id.0].update(rule, new_ctx, alloc)
                .map_err(|_| AdaptError::InvalidRuleMutation(rule.name))?);
        }

        Ok((s, new_ctx))
    }

    pub fn new_with(
        grammar: &'arn GrammarFile<'arn, 'grm>,
        alloc: Allocs<'arn>,
    ) -> (Self, VarMap<'arn, 'grm>) {
        // We create a new grammar by adapting an empty one
        GrammarState::new()
            .adapt_with(grammar, VarMap::default(), None, alloc)
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

#[derive(Copy, Clone)]
pub struct RuleState<'arn, 'grm> {
    pub name: &'grm str,
    pub args: &'arn [&'grm str],
    pub blocks: &'arn [BlockState<'arn, 'grm>],
}

pub enum UpdateError {
    ToposortCycle,
}

impl<'arn, 'grm> RuleState<'arn, 'grm> {
    pub fn new_empty(name: &'grm str, args: &'arn [&'grm str]) -> Self {
        Self {
            name,
            blocks: &[],
            args,
        }
    }

    pub fn update(
        &self,
        r: &'arn Rule<'arn, 'grm>,
        ctx: VarMap<'arn, 'grm>,
        allocs: Allocs<'arn>
    ) -> Result<Self, UpdateError> {
        assert_eq!(self.name, r.name);
        assert_eq!(self.args, r.args);

        //TODO remove this allocation?
        let new_nodes: HashSet<&'grm str> = r.blocks.iter().map(|n| n.0).collect();
        let mut result = Vec::with_capacity(self.blocks.len() + r.blocks.len());
        let mut new_iter = r.blocks.iter();

        for old_block in self.blocks {
            // If this block is only present in the old rule, take it first
            if !new_nodes.contains(old_block.name) {
                result.push(*old_block);
                continue
            }

            // Take all rules from the new rule until we found the matching block
            loop {
                // Add all blocks that don't match
                let Some(new_block) = new_iter.next() else {
                    return Err(UpdateError::ToposortCycle)
                };
                if new_block.0 != old_block.name {
                    result.push(BlockState::new(new_block, ctx, allocs));
                    continue;
                }

                // Merge blocks
                result.push(old_block.update(new_block, ctx, allocs));
                break
            }
        }

        // Add remaining blocks from new iter
        for new_block in new_iter {
            result.push(BlockState::new(new_block, ctx, allocs));
        }

        Ok(Self {
            name: self.name,
            args: self.args,
            blocks: allocs.alloc_extend(result),
        })
    }
}

#[derive(Copy, Clone)]
pub struct BlockState<'arn, 'grm> {
    pub name: &'grm str,
    pub constructors: &'arn [Constructor<'arn, 'grm>],
}

pub type Constructor<'arn, 'grm> = (&'arn AnnotatedRuleExpr<'arn, 'grm>, VarMap<'arn, 'grm>);

impl<'arn, 'grm> BlockState<'arn, 'grm> {
    pub fn new(block: &'arn Block<'arn, 'grm>, ctx: VarMap<'arn, 'grm>, allocs: Allocs<'arn>) -> Self {
        Self {
            name: block.0,
            constructors: allocs.alloc_extend(block.1.iter().map(|r| (r, ctx))),
        }
    }

    #[must_use]
    pub fn update(&self, b: &'arn Block<'arn, 'grm>, ctx: VarMap<'arn, 'grm>, allocs: Allocs<'arn>) -> Self {
        assert_eq!(self.name, b.0);
        Self {
            name: self.name,
            constructors: allocs.alloc_extend_len(self.constructors.len() + b.1.len(), self.constructors.iter().cloned().chain(b.1.iter().map(|r| (r, ctx)))),
        }
    }
}
