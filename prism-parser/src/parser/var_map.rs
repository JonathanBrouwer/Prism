use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use by_address::ByAddress;
use itertools::Itertools;
use crate::core::adaptive::{BlockState, GrammarState, RuleId};
use crate::core::context::{ParserContext, PR};
use crate::core::cow::Cow;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::state::PState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::RuleExpr;
use crate::parser::parser_rule_expr::parser_expr;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VarMap<'arn, 'grm>(HashMap<&'grm str, VarMapValue<'arn, 'grm>>);

impl<'arn, 'grm> VarMap<'arn, 'grm> {
    pub fn get<'a>(&'a self, k: &str) -> Option<&'a VarMapValue<'arn, 'grm>> {
        self.0.get(k)
    }

    pub fn iter(&self) -> impl Iterator<Item=(&'grm str, &VarMapValue<'arn, 'grm>)> {
        self.0.iter().map(|(k, v)| (*k, v))
    }

    pub fn extend<T: IntoIterator<Item = (&'grm str, VarMapValue<'arn, 'grm>)>>(&mut self, iter: T) {
        self.0.extend(iter)
    }
}

impl<'arn, 'grm> FromIterator<(&'grm str, VarMapValue<'arn, 'grm>)> for VarMap<'arn, 'grm> {
    fn from_iter<T: IntoIterator<Item=(&'grm str, VarMapValue<'arn, 'grm>)>>(iter: T) -> Self {
        Self(HashMap::from_iter(iter))
    }
}

impl<'arn, 'grm> Hash for VarMap<'arn, 'grm> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (k, v) in self.0.iter().sorted_by_key(|(k, _v)| **k) {
            k.hash(state);
            v.hash(state);
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum VarMapValue<'arn, 'grm> {
    Expr {
        expr: &'arn RuleExpr<'grm, RuleAction<'arn, 'grm>>,
        blocks: ByAddress<&'arn [BlockState<'arn, 'grm>]>,
        rule_args: VarMap<'arn, 'grm>,
        vars: VarMap<'arn, 'grm>,
    },
    Value(Cow<'arn, ActionResult<'arn, 'grm>>),
}

impl<'arn, 'grm> VarMapValue<'arn, 'grm> {
    pub fn as_rule(&self) -> Option<RuleId> {
        let VarMapValue::Value(v) = self else {
            return None;
        };
        let ActionResult::RuleRef(rule) = v.as_ref() else {
            return None;
        };
        Some(*rule)
    }

    pub fn as_value(&self) -> Option<&Cow<'arn, ActionResult<'arn, 'grm>>> {
        if let VarMapValue::Value(value) = self {
            Some(value)
        } else {
            None
        }
    }
}