use std::fmt::{Display, Formatter};
use crate::core::adaptive::{BlockState, GrammarState, RuleId};
use crate::core::cow::Cow;
use crate::grammar::RuleExpr;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;
use by_address::ByAddress;
use std::hash::Hash;
use typed_arena::Arena;
use crate::core::context::ParserContext;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::state::PState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::parser::parser_rule_expr::parser_expr;

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct VarMap<'arn, 'grm>(Option<ByAddress<&'arn VarMapNode<'arn, 'grm>>>);

#[derive(Clone, Eq, PartialEq, Hash)]
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

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct CapturedExpr<'arn, 'grm> {
    pub expr: &'arn RuleExpr<'grm, RuleAction<'arn, 'grm>>,
    pub blocks: ByAddress<&'arn [BlockState<'arn, 'grm>]>,
    pub rule_args: VarMap<'arn, 'grm>,
    pub vars: VarMap<'arn, 'grm>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
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
    
    pub fn run_to_ar<'a, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(&'a self, rules: &'arn GrammarState<'arn, 'grm>, state: &mut PState<'arn, 'grm, E>, context: ParserContext) -> Option<Cow<'a, ActionResult<'arn, 'grm>>> {
        Some(match self {
            VarMapValue::Expr(captured_expr) => {
                parser_expr(rules, &captured_expr.blocks, &captured_expr.expr, captured_expr.rule_args, captured_expr.vars).parse(Pos::invalid(), state, context).ok()?.rtrn
            }
            VarMapValue::RuleId(_) => panic!("Not an ar"),
            VarMapValue::Value(v) => {
                v.clone()
            },
        })
    }

    pub fn as_rule_id(&self) -> Option<RuleId> {
        if let VarMapValue::RuleId(rule) = self {
            Some(*rule)
        } else {
            None
        }
    }
}

impl<'arn, 'grm> Display for VarMap<'arn, 'grm> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Displaying varmap (top is nearest)")?;
        for (k, v) in self.iter() {
            writeln!(f, "{k} = {v}")?;
        }
        Ok(())
    }
}

impl<'arn, 'grm> Display for VarMapValue<'arn, 'grm> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VarMapValue::Expr(_) => write!(f, "Unevaluted expression"),
            VarMapValue::RuleId(i) => write!(f, "Rule with id {i}"),
            VarMapValue::Value(v) => write!(f, "{v:?}"),
        }
    }
}