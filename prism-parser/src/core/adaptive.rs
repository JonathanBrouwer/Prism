use std::any::type_name;
use crate::core::cache::Allocs;
use crate::core::pos::Pos;
use crate::grammar::{AnnotatedRuleExpr, Block, GrammarFile, Rule};
use crate::parser::var_map::{VarMap, VarMapValue};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::iter;
use crate::core::parsable::Parsable;

#[derive(Copy, Clone)]
pub struct GrammarState<'arn, 'grm> {
    rules: &'arn [&'arn RuleState<'arn, 'grm>],
    last_mut_pos: Option<Pos>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct RuleId(usize);

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for RuleId {
    fn from_rule(rule: RuleId, _allocs: Allocs<'arn>) -> Self {
        rule
    }
}

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
            rules: &[],
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
        // If we already tried to adapt at this position before, crash to prevent infinite loop
        if let Some(pos) = pos {
            if let Some(last_mut_pos) = self.last_mut_pos {
                if pos == last_mut_pos {
                    return Err(AdaptError::SamePos(pos));
                }
            }
        }

        // Create a new ruleid or find an existing rule id for each rule that is adopted
        let mut new_rules = self.rules.to_vec();
        let mut new_ctx = ctx;
        let tmp: Vec<_> = grammar
            .rules
            .iter()
            .map(|new_rule| {
                let rule = if new_rule.adapt {
                    let value = ctx.get(new_rule.name).expect("Name exists in context");
                    *value.as_value().expect("Var map value is value").into_value::<RuleId>()
                } else {
                    new_rules.push(alloc.alloc(RuleState::new_empty(new_rule.name, new_rule.args)));
                    RuleId(new_rules.len() - 1)
                };
                new_ctx = new_ctx.insert(
                    new_rule.name,
                    VarMapValue::Value(alloc.alloc(rule).to_parsed()),
                    alloc
                );
                (new_rule.name, rule)
            })
            .collect();

        // Update each rule that is to be adopted, stored in `result`
        for (&(_, id), rule) in tmp.iter().zip(grammar.rules.iter()) {
            new_rules[id.0] = alloc.alloc(
                new_rules[id.0]
                    .update(rule, new_ctx, alloc)
                    .map_err(|_| AdaptError::InvalidRuleMutation(rule.name))?,
            );
        }

        Ok((
            Self {
                last_mut_pos: pos,
                rules: alloc.alloc_extend(new_rules),
            },
            new_ctx,
        ))
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

pub type ArgsSlice<'arn, 'grm> = &'arn [(&'grm str, &'grm str)];

#[derive(Copy, Clone)]
pub struct RuleState<'arn, 'grm> {
    pub name: &'grm str,
    pub args: ArgsSlice<'arn, 'grm>,
    pub blocks: &'arn [BlockState<'arn, 'grm>],
}

pub enum UpdateError {
    ToposortCycle,
}

impl<'arn, 'grm> RuleState<'arn, 'grm> {
    pub fn new_empty(name: &'grm str, args: ArgsSlice<'arn, 'grm>) -> Self {
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
        allocs: Allocs<'arn>,
    ) -> Result<Self, UpdateError> {
        assert_eq!(self.name, r.name);
        assert_eq!(self.args, r.args);

        //TODO remove this allocation?
        let mut result = Vec::with_capacity(self.blocks.len() + r.blocks.len());
        let mut old_iter = self.blocks.iter();

        for new_block in r.blocks {
            // If this new block should not match an old block, add it as a new block state
            if !new_block.adapt {
                result.push(BlockState::new(new_block, ctx, allocs));
                continue;
            }
            // Find matching old block
            loop {
                // If the matching block can't be found, it must've already occurred, and therefore we have a cycle
                let Some(old_block) = old_iter.next() else {
                    return Err(UpdateError::ToposortCycle);
                };
                // If this is not the matching block, add it and continue searching
                if old_block.name != new_block.name {
                    result.push(*old_block);
                    continue;
                }
                result.push(old_block.update(new_block, ctx, allocs));
                break;
            }
        }
        for old_block in old_iter {
            result.push(*old_block);
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
    pub fn new(
        block: &'arn Block<'arn, 'grm>,
        ctx: VarMap<'arn, 'grm>,
        allocs: Allocs<'arn>,
    ) -> Self {
        Self {
            name: block.name,
            constructors: allocs.alloc_extend(block.constructors.iter().map(|r| (r, ctx))),
        }
    }

    #[must_use]
    pub fn update(
        &self,
        b: &'arn Block<'arn, 'grm>,
        ctx: VarMap<'arn, 'grm>,
        allocs: Allocs<'arn>,
    ) -> Self {
        assert_eq!(self.name, b.name);
        Self {
            name: self.name,
            constructors: allocs.alloc_extend_len(
                self.constructors.len() + b.constructors.len(),
                self.constructors
                    .iter()
                    .cloned()
                    .chain(b.constructors.iter().map(|r| (r, ctx))),
            ),
        }
    }
}
