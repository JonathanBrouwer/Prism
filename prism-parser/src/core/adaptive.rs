use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::pos::Pos;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::grammar::grammar_file::GrammarFile;
use crate::grammar::identifier::Identifier;
use crate::grammar::rule::Rule;
use crate::grammar::rule_block::RuleBlock;
use crate::parsable::ParseResult;
use crate::parser::VarMap;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone)]
pub struct GrammarState<'arn> {
    rules: &'arn [&'arn RuleState<'arn>],
    last_mut_pos: Option<Pos>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct RuleId(usize);

impl ParseResult for RuleId {}

impl Display for RuleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for GrammarState<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum AdaptError {
    InvalidRuleMutation(Identifier),
    SamePos(Pos),
}

impl<'arn> GrammarState<'arn> {
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
        grammar: &'arn GrammarFile<'arn>,
        ctx: VarMap<'arn>,
        pos: Option<Pos>,
        alloc: Allocs<'arn>,
        input_table: &InputTable<'arn>,
    ) -> Result<(Self, VarMap<'arn>), AdaptError> {
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
                let new_rule_name = new_rule.name.as_str(input_table);

                let rule = if new_rule.adapt {
                    let value = ctx.get(new_rule_name).expect("Name exists in context");
                    *value.into_value::<RuleId>()
                } else {
                    new_rules.push(alloc.alloc(RuleState::new_empty(new_rule.name, new_rule.args)));
                    RuleId(new_rules.len() - 1)
                };
                new_ctx = new_ctx.insert(new_rule_name, alloc.alloc(rule).to_parsed(), alloc);
                (new_rule.name, rule)
            })
            .collect();

        // Update each rule that is to be adopted, stored in `result`
        for (&(_, id), rule) in tmp.iter().zip(grammar.rules.iter()) {
            new_rules[id.0] = alloc.alloc(
                new_rules[id.0]
                    .update(rule, new_ctx, alloc, input_table)
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
        grammar: &'arn GrammarFile<'arn>,
        alloc: Allocs<'arn>,
        input_table: &InputTable<'arn>,
    ) -> (Self, VarMap<'arn>) {
        // We create a new grammar by adapting an empty one
        GrammarState::new()
            .adapt_with(grammar, VarMap::default(), None, alloc, input_table)
            .expect("")
    }

    pub fn get(&self, rule: RuleId) -> Option<&RuleState<'arn>> {
        self.rules.get(rule.0).map(|v| &**v)
    }

    pub fn unique_id(&self) -> GrammarStateId {
        GrammarStateId(self.rules.as_ptr() as usize)
    }
}

// TODO instead of one global GrammarStateId, we can track this per rule. Create a graph of rule ids and update the id when one of its components changes
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct GrammarStateId(usize);

pub type ArgsSlice<'arn> = &'arn [(Identifier, Identifier)];

#[derive(Copy, Clone)]
pub struct RuleState<'arn> {
    pub name: Identifier,
    pub args: ArgsSlice<'arn>,
    pub blocks: &'arn [BlockState<'arn>],
}

pub enum UpdateError {
    ToposortCycle,
}

impl<'arn> RuleState<'arn> {
    pub fn new_empty(name: Identifier, args: ArgsSlice<'arn>) -> Self {
        Self {
            name,
            blocks: &[],
            args,
        }
    }

    pub fn update(
        &self,
        r: &'arn Rule<'arn>,
        ctx: VarMap<'arn>,
        allocs: Allocs<'arn>,
        input_table: &InputTable<'arn>,
    ) -> Result<Self, UpdateError> {
        assert_eq!(self.name.as_str(input_table), r.name.as_str(input_table));
        for (a1, a2) in self.args.iter().zip(r.args) {
            assert_eq!(a1.0.as_str(input_table), a2.0.as_str(input_table));
            assert_eq!(a1.1.as_str(input_table), a2.1.as_str(input_table));
        }

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
                if old_block.name.as_str(input_table) != new_block.name.as_str(input_table) {
                    result.push(*old_block);
                    continue;
                }
                result.push(old_block.update(new_block, ctx, allocs, input_table));
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
pub struct BlockState<'arn> {
    pub name: Identifier,
    pub constructors: &'arn [Constructor<'arn>],
}

pub type Constructor<'arn> = (&'arn AnnotatedRuleExpr<'arn>, VarMap<'arn>);

impl<'arn> BlockState<'arn> {
    pub fn new(block: &'arn RuleBlock<'arn>, ctx: VarMap<'arn>, allocs: Allocs<'arn>) -> Self {
        Self {
            name: block.name,
            constructors: allocs.alloc_extend(block.constructors.iter().map(|r| (r, ctx))),
        }
    }

    #[must_use]
    pub fn update(
        &self,
        b: &'arn RuleBlock<'arn>,
        ctx: VarMap<'arn>,
        allocs: Allocs<'arn>,
        input_table: &InputTable<'arn>,
    ) -> Self {
        assert_eq!(self.name.as_str(input_table), b.name.as_str(input_table));
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
