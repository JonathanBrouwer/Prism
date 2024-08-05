use crate::core::adaptive::{BlockState, GrammarState, RuleId};
use crate::core::context::ParserContext;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::state::PState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::RuleExpr;
use crate::parser::parser_rule_expr::parser_expr;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;
use by_address::ByAddress;
use std::fmt::{Debug, Formatter};
use std::iter;
use std::ptr::null;
use crate::core::cache::Allocs;

#[derive(Default, Copy, Clone)]
pub struct VarMap<'arn, 'grm>(Option<&'arn VarMapNode<'arn, 'grm>>);

impl<'arn, 'grm> Debug for VarMap<'arn, 'grm> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Printing varmap:")?;
        for (name, value) in self.iter_cloned() {
            writeln!(f, "- {name}: {value:?}")?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct VarMapNode<'arn, 'grm> {
    next: Option<&'arn Self>,
    key: &'arn str,
    value: VarMapValue<'arn, 'grm>,
}

pub struct VarMapIterator<'arn, 'grm> {
    current: Option<&'arn VarMapNode<'arn, 'grm>>,
}

impl<'arn, 'grm> Iterator for VarMapIterator<'arn, 'grm> {
    type Item = (&'arn str, VarMapValue<'arn, 'grm>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => None,
            Some(node) => {
                self.current = node.next;
                Some((node.key, node.value.clone()))
            }
        }
    }
}

impl<'arn, 'grm> VarMap<'arn, 'grm> {
    pub fn get<'a>(&'a self, k: &str) -> Option<&'a VarMapValue<'arn, 'grm>> {
        let mut node = self.0?;
        loop {
            if node.key == k {
                return Some(&node.value);
            }
            node = node.next?;
        }
    }

    pub fn iter_cloned(&self) -> impl Iterator<Item = (&'arn str, VarMapValue<'arn, 'grm>)> {
        VarMapIterator {
            current: self.0,
        }
    }

    #[must_use]
    pub fn insert(
        self,
        key: &'arn str,
        value: VarMapValue<'arn, 'grm>,
        alloc: Allocs<'arn>,
    ) -> Self {
        self.extend(iter::once((key, value)), alloc)
    }

    #[must_use]
    pub fn extend<T: IntoIterator<Item = (&'arn str, VarMapValue<'arn, 'grm>)>>(
        mut self,
        iter: T,
        alloc: Allocs<'arn>,
    ) -> Self {
        for (key, value) in iter {
            self.0 = Some(alloc.alloc(VarMapNode {
                next: self.0,
                key,
                value,
            }))
        }
        self
    }

    pub fn from_iter<T: IntoIterator<Item = (&'arn str, VarMapValue<'arn, 'grm>)>>(
        iter: T,
        alloc: Allocs<'arn>,
    ) -> Self {
        let s = Self::default();
        s.extend(iter, alloc)
    }
    
    pub fn as_ptr(&self) -> *const VarMapNode {
        self.0.map(|r| r as *const VarMapNode).unwrap_or(null())
    }
}

pub type BlockCtx<'arn, 'grm> = (
    &'arn [BlockState<'arn, 'grm>],
    VarMap<'arn, 'grm>,
);

#[derive(Copy, Clone)]
pub struct CapturedExpr<'arn, 'grm> {
    pub expr: &'arn RuleExpr<'arn, 'grm>,
    pub block_ctx: BlockCtx<'arn, 'grm>,
    pub vars: VarMap<'arn, 'grm>,
}

#[derive(Copy, Clone)]
pub enum VarMapValue<'arn, 'grm> {
    Expr(CapturedExpr<'arn, 'grm>),
    Value(&'arn ActionResult<'arn, 'grm>),
}

impl<'arm, 'grm> Debug for VarMapValue<'arm, 'grm> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VarMapValue::Expr(_) => write!(f, "{{expr}}"),
            VarMapValue::Value(ar) => write!(f, "{ar:?}"),
        }
    }
}

impl<'arn, 'grm> VarMapValue<'arn, 'grm> {
    pub fn new_rule(rule: RuleId, alloc: Allocs<'arn>) -> Self {
        Self::Value(alloc.alloc(ActionResult::RuleId(rule)))
    }

    pub fn as_value(&self) -> Option<&ActionResult<'arn, 'grm>> {
        if let VarMapValue::Value(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn run_to_ar<'a, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
        &'a self,
        rules: &'arn GrammarState<'arn, 'grm>,
        state: &mut PState<'arn, 'grm, E>,
        context: ParserContext,
    ) -> Option<&'arn ActionResult<'arn, 'grm>> {
        Some(match self {
            VarMapValue::Expr(captured_expr) => {
                parser_expr(
                    rules,
                    captured_expr.block_ctx,
                    captured_expr.expr,
                    captured_expr.vars,
                )
                .parse(Pos::invalid(), state, context)
                .ok()?
                .rtrn
            }
            VarMapValue::Value(v) => v,
        })
    }

    pub fn as_rule_id(&self) -> Option<RuleId> {
        let VarMapValue::Value(ar) = self else {
            return None;
        };
        let ActionResult::RuleId(rule) = ar else {
            return None;
        };
        Some(*rule)
    }
}
