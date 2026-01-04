use crate::core::allocs::alloc_extend;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::grammar::grammar_file::GrammarFile;
use crate::grammar::rule::Rule;
use crate::grammar::rule_block::RuleBlock;
use crate::parsable::parsed::ArcExt;
use crate::parser::VarMap;
use prism_input::input::Input;
use prism_input::input_table::InputTable;
use prism_input::pos::Pos;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

pub struct GrammarState {
    rules: Arc<[Arc<RuleState>]>,
    last_mut_pos: Option<Pos>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct RuleId(usize);

impl Display for RuleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for GrammarState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum AdaptError {
    InvalidRuleMutation(Input),
    SamePos(Pos),
}

impl GrammarState {
    pub fn new() -> Self {
        Self {
            rules: Arc::new([]),
            last_mut_pos: None,
        }
    }

    /// Adapt this grammar state with `grammar`.
    /// To unify rules, use the names of the rules in `ctx`
    pub fn adapt_with(
        &self,
        grammar: &GrammarFile,
        ctx: &VarMap,
        pos: Option<Pos>,
        input_table: &InputTable,
    ) -> Result<(Self, VarMap), AdaptError> {
        // If we already tried to adapt at this position before, crash to prevent infinite loop
        if let Some(pos) = pos
            && let Some(last_mut_pos) = self.last_mut_pos
            && pos == last_mut_pos
        {
            return Err(AdaptError::SamePos(pos));
        }

        // Create a new ruleid or find an existing rule id for each rule that is adopted
        let mut new_rules = self.rules.iter().cloned().collect::<Vec<_>>();
        let mut new_ctx = ctx.clone();
        let tmp: Vec<_> = grammar
            .rules
            .iter()
            .map(|new_rule| {
                let rule = if new_rule.adapt {
                    let value = ctx
                        .get(new_rule.name.as_str(input_table))
                        .expect("Name exists in context");
                    *value.value_ref::<RuleId>()
                } else {
                    new_rules.push(Arc::new(RuleState::new_empty(
                        new_rule.name.clone(),
                        new_rule.args.clone(),
                    )));
                    RuleId(new_rules.len() - 1)
                };
                new_ctx = new_ctx.insert(
                    new_rule.name.as_str(input_table).to_string(),
                    Arc::new(rule).to_parsed(),
                );
                rule
            })
            .collect();

        // Update each rule that is to be adopted, stored in `result`
        for (&id, rule) in tmp.iter().zip(grammar.rules.iter()) {
            new_rules[id.0] = Arc::new(
                new_rules[id.0]
                    .update(rule, new_ctx.clone(), input_table)
                    .map_err(|_| AdaptError::InvalidRuleMutation(rule.name.clone()))?,
            );
        }

        Ok((
            Self {
                last_mut_pos: pos,
                rules: new_rules.into(),
            },
            new_ctx,
        ))
    }

    pub fn new_with(grammar: &GrammarFile, input_table: &InputTable) -> (Self, VarMap) {
        // We create a new grammar by adapting an empty one
        GrammarState::new()
            .adapt_with(grammar, &VarMap::default(), None, input_table)
            .expect("")
    }

    pub fn get(&self, rule: RuleId) -> Option<&RuleState> {
        self.rules.get(rule.0).map(|v| &**v)
    }

    pub fn unique_id(&self) -> GrammarStateId {
        GrammarStateId(self.rules.as_ptr() as usize)
    }
}

// TODO instead of one global GrammarStateId, we can track this per rule. Create a graph of rule ids and update the id when one of its components changes
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct GrammarStateId(usize);

pub type ArgsSlice = Arc<[(Input, Input)]>;

pub struct RuleState {
    pub name: Input,
    pub args: ArgsSlice,
    pub blocks: Arc<[Arc<BlockState>]>,
}

pub enum UpdateError {
    ToposortCycle,
}

impl RuleState {
    pub fn new_empty(name: Input, args: ArgsSlice) -> Self {
        Self {
            name,
            blocks: vec![].into(),
            args,
        }
    }

    pub fn update(
        &self,
        r: &Rule,
        ctx: VarMap,

        input_table: &InputTable,
    ) -> Result<Self, UpdateError> {
        assert_eq!(self.name.as_str(input_table), r.name.as_str(input_table));
        for (a1, a2) in self.args.iter().zip(r.args.iter()) {
            assert_eq!(a1.0.as_str(input_table), a2.0.as_str(input_table));
            assert_eq!(a1.1.as_str(input_table), a2.1.as_str(input_table));
        }

        let mut result: Vec<Arc<BlockState>> =
            Vec::with_capacity(self.blocks.len() + r.blocks.len());
        let mut old_iter = self.blocks.iter();

        for new_block in r.blocks.iter() {
            // If this new block should not match an old block, add it as a new block state
            if !new_block.adapt {
                result.push(Arc::new(BlockState::new(new_block, ctx.clone())));
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
                    result.push(old_block.clone());
                    continue;
                }
                result.push(old_block.update(new_block, ctx.clone(), input_table));
                break;
            }
        }
        for old_block in old_iter {
            result.push(old_block.clone());
        }

        Ok(Self {
            name: self.name.clone(),
            args: self.args.clone(),
            blocks: result.into(),
        })
    }
}

#[derive(Clone)]
pub struct BlockState {
    pub name: Input,
    pub constructors: Arc<[Constructor]>,
}

pub type Constructor = (Arc<AnnotatedRuleExpr>, VarMap);

impl BlockState {
    pub fn new(block: &RuleBlock, ctx: VarMap) -> Self {
        Self {
            name: block.name.clone(),
            constructors: alloc_extend(block.constructors.iter().map(|r| (r.clone(), ctx.clone()))),
        }
    }

    #[must_use]
    pub fn update(&self, b: &RuleBlock, ctx: VarMap, input_table: &InputTable) -> Arc<Self> {
        assert_eq!(self.name.as_str(input_table), b.name.as_str(input_table));
        Arc::new(Self {
            name: self.name.clone(),
            constructors: alloc_extend(
                self.constructors
                    .iter()
                    .cloned()
                    .chain(b.constructors.iter().map(|r| (r.clone(), ctx.clone()))),
            ),
        })
    }
}
