use std::hash::{Hash, Hasher};
use std::iter;
use by_address::ByAddress;
use itertools::Itertools;
use crate::core::adaptive::{BlockState, GrammarState, RuleId};
use crate::core::cache::Allocs;
use crate::core::cow::Cow;
use crate::grammar::RuleExpr;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct VarMap<'arn, 'grm>(Option<ByAddress<&'arn VarMapNode<'arn, 'grm>>>);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct VarMapNode<'arn, 'grm> {
    next: Option<&'arn Self>,
    key: &'grm str,
    value: VarMapValue<'arn, 'grm>,
}

impl<'arn, 'grm> VarMap<'arn, 'grm> {
    pub fn get<'a>(&'a self, k: &str) -> Option<&'a VarMapValue<'arn, 'grm>> {
        todo!()
        // self.0.get(k)
    }

    pub fn iter(&self) -> impl Iterator<Item=(&'grm str, &VarMapValue<'arn, 'grm>)> {
        todo!();
        return iter::empty();
        // self.0.iter().map(|(k, v)| (*k, v))
    }

    pub fn extend<T: IntoIterator<Item = (&'grm str, VarMapValue<'arn, 'grm>)>>(&mut self, iter: T, alloc: &Allocs<'arn, 'grm>) {
        todo!()
        // self.0.extend(iter)
    }

    pub fn from_iter<T: IntoIterator<Item=(&'grm str, VarMapValue<'arn, 'grm>)>>(iter: T, alloc: &Allocs<'arn, 'grm>) -> Self {
        todo!()
        // Self(RedBlackTreeMap::from_iter(iter))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CapturedExpr<'arn, 'grm> {
    pub expr: &'arn RuleExpr<'grm, RuleAction<'arn, 'grm>>,
    pub blocks: ByAddress<&'arn [BlockState<'arn, 'grm>]>,
    pub rule_args: VarMap<'arn, 'grm>,
    pub vars: VarMap<'arn, 'grm>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum VarMapValue<'arn, 'grm> {
    Expr(CapturedExpr<'arn, 'grm>),
    RuleId(RuleId),
    Value(Cow<'arn, ActionResult<'arn, 'grm>>),
}

impl<'arn, 'grm> VarMapValue<'arn, 'grm> {
    pub fn as_value(&self) -> Option<&Cow<'arn, ActionResult<'arn, 'grm>>> {
        if let VarMapValue::Value(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_rule_id(&self) -> Option<RuleId> {
        if let VarMapValue::RuleId(rule) = self {
            Some(*rule)
        } else {
            None
        }
    }
}