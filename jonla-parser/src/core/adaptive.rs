use crate::core::context::{Raw, RawEnv};
use crate::core::toposet::TopoSet;
use crate::grammar::{AnnotatedRuleExpr, Block, GrammarFile, Rule};
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::apply_action::apply_rawenv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::mem;
use std::sync::Arc;

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

impl<'b, 'grm> Default for GrammarState<'b, 'grm> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'b, 'grm> GrammarState<'b, 'grm> {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn with(
        &self,
        grammar: &'b GrammarFile<'grm>,
        ctx: &HashMap<&'grm str, Arc<RawEnv<'b, 'grm>>>,
    ) -> Result<(Self, impl Iterator<Item = (&'grm str, RuleId)> + 'b), &'grm str> {
        let mut s = Self {
            rules: self.rules.clone(),
        };

        let mut result = vec![];
        for new_rule in &grammar.rules {
            if let Some(ar) = ctx.get(new_rule.name) {
                if let ActionResult::RuleRef(rule) = apply_rawenv(ar) {
                    result.push((new_rule.name, rule))
                } else {
                    panic!("Tried to run variable `{}` as a rule, but it does not refer to a rule. {ar:?}", new_rule.name);
                }
            } else {
                result.push((new_rule.name, RuleId(s.rules.len())));
                s.rules
                    .push(RuleState::new_empty(new_rule.name, &new_rule.args));
            };
        }

        let ctx = Arc::new(
            Iterator::chain(
                ctx.iter().map(|(&k, v)| (k, v.clone())),
                result
                    .iter()
                    .map(|&(k, v)| (k, Arc::new(RawEnv::from_raw(Raw::Rule(v))))),
            )
            .collect::<HashMap<_, _>>(),
        );

        for (&(_, id), rule) in result.iter().zip(grammar.rules.iter()) {
            s.rules[id.0].update(rule, &ctx).map_err(|_| rule.name)?;
        }

        Ok((s, result.into_iter()))
    }

    pub fn new_with(
        grammar: &'b GrammarFile<'grm>,
    ) -> (Self, impl Iterator<Item = (&'grm str, RuleId)> + 'b) {
        GrammarState::new().with(grammar, &HashMap::new()).unwrap()
    }

    pub fn get(&self, rule: RuleId) -> Option<&RuleState<'b, 'grm>> {
        self.rules.get(rule.0)
    }
}

#[derive(Clone)]
pub struct RuleState<'b, 'grm> {
    pub name: &'grm str,
    pub blocks: Vec<BlockState<'b, 'grm>>,
    order: TopoSet<'grm>,
    pub arg_names: &'b Vec<&'grm str>,
}

impl<'b, 'grm> RuleState<'b, 'grm> {
    pub fn new_empty(name: &'grm str, arg_names: &'b Vec<&'grm str>) -> Self {
        Self {
            name,
            blocks: Vec::new(),
            order: TopoSet::new(),
            arg_names,
        }
    }

    pub fn update(
        &mut self,
        r: &'b Rule<'grm>,
        ctx: &Arc<HashMap<&'grm str, Arc<RawEnv<'b, 'grm>>>>,
    ) -> Result<(), ()> {
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
pub struct BlockState<'b, 'grm> {
    pub name: &'grm str,
    pub constructors: Vec<(
        &'b AnnotatedRuleExpr<'grm>,
        Arc<HashMap<&'grm str, Arc<RawEnv<'b, 'grm>>>>,
    )>,
}

impl<'b, 'grm> BlockState<'b, 'grm> {
    pub fn new(
        block: &'b Block<'grm>,
        ctx: &Arc<HashMap<&'grm str, Arc<RawEnv<'b, 'grm>>>>,
    ) -> Self {
        Self {
            name: block.0,
            constructors: block.1.iter().map(|r| (r, ctx.clone())).collect(),
        }
    }

    pub fn update(
        &mut self,
        b: &'b Block<'grm>,
        ctx: &Arc<HashMap<&'grm str, Arc<RawEnv<'b, 'grm>>>>,
    ) {
        assert_eq!(self.name, b.0);
        self.constructors
            .extend(b.1.iter().map(|r| (r, ctx.clone())));
    }
}
