use crate::core::adaptive::{BlockState, RuleId};
use crate::core::cow::Cow;
use crate::grammar::RuleExpr;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;
use by_address::ByAddress;
use std::hash::Hash;
use typed_arena::Arena;

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct VarMap<'arn, 'grm>(Option<ByAddress<&'arn VarMapNode<'arn, 'grm>>>);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VarMapNode<'arn, 'grm> {
    next: Option<&'arn Self>,
    key: &'grm str,
    value: VarMapValue<'arn, 'grm>,
}

pub struct VarMapIterator<'arn, 'grm> {
    current: Option<&'arn VarMapNode<'arn, 'grm>>,
}

impl<'arn, 'grm> Iterator for VarMapIterator<'arn, 'grm> {
    type Item = (&'grm str, &'arn VarMapValue<'arn, 'grm>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => None,
            Some(node) => {
                self.current = node.next;
                Some((node.key, &node.value))
            }
        }
    }
}

impl<'arn, 'grm> VarMap<'arn, 'grm> {
    pub fn get<'a>(&'a self, k: &str) -> Option<&'a VarMapValue<'arn, 'grm>> {
        let mut node = *self.0?;
        loop {
            if node.key == k {
                return Some(&node.value);
            }
            node = node.next?;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'grm str, &'arn VarMapValue<'arn, 'grm>)> {
        VarMapIterator {
            current: self.0.map(|v| *v),
        }
    }

    #[must_use]
    pub fn extend<T: IntoIterator<Item = (&'grm str, VarMapValue<'arn, 'grm>)>>(
        mut self,
        iter: T,
        alloc: &'arn Arena<VarMapNode<'arn, 'grm>>,
    ) -> Self {
        for (key, value) in iter {
            self.0 = Some(ByAddress(alloc.alloc(VarMapNode {
                next: self.0.map(|v| *v),
                key,
                value,
            })))
        }
        self
    }

    pub fn from_iter<T: IntoIterator<Item = (&'grm str, VarMapValue<'arn, 'grm>)>>(
        iter: T,
        alloc: &'arn Arena<VarMapNode<'arn, 'grm>>,
    ) -> Self {
        let s = Self::default();
        s.extend(iter, alloc)
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
