use crate::core::adaptive::{BlockState, GrammarState};
use crate::core::cache::Allocs;
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::state::ParserState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::rule_expr::RuleExpr;
use crate::parsable::parsed::Parsed;
use std::fmt::{Debug, Formatter};
use std::iter;
use std::ptr::null;

#[derive(Default, Copy, Clone)]
pub struct VarMap<'arn, 'grm>(Option<&'arn VarMapNode<'arn, 'grm>>);

impl Debug for VarMap<'_, '_> {
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
                Some((node.key, node.value))
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
        VarMapIterator { current: self.0 }
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

pub type BlockCtx<'arn, 'grm> = (&'arn [BlockState<'arn, 'grm>], VarMap<'arn, 'grm>);

#[derive(Copy, Clone)]
pub struct CapturedExpr<'arn, 'grm> {
    pub expr: &'arn RuleExpr<'arn, 'grm>,
    pub block_ctx: BlockCtx<'arn, 'grm>,
    pub vars: VarMap<'arn, 'grm>,
}

#[derive(Copy, Clone)]
pub enum VarMapValue<'arn, 'grm> {
    Expr(CapturedExpr<'arn, 'grm>),
    Value(Parsed<'arn, 'grm>),
}

impl Debug for VarMapValue<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VarMapValue::Expr(_) => write!(f, "{{expr}}"),
            VarMapValue::Value(_) => write!(f, "{{value}}"),
        }
    }
}

impl<'arn, 'grm> VarMapValue<'arn, 'grm> {
    pub fn as_value(&self) -> Option<Parsed<'arn, 'grm>> {
        if let VarMapValue::Value(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn run_to_value<'a, E: ParseError<L = ErrorLabel<'grm>>>(
        &'a self,
        rules: &'arn GrammarState<'arn, 'grm>,
        state: &mut ParserState<'arn, 'grm, E>,
        context: ParserContext,
    ) -> Option<Parsed<'arn, 'grm>> {
        Some(match self {
            VarMapValue::Expr(captured_expr) => {
                state
                    .parse_expr(
                        rules,
                        captured_expr.block_ctx,
                        captured_expr.expr,
                        captured_expr.vars,
                        Pos::invalid(),
                        context,
                    )
                    .ok()?
                    .rtrn
            }
            VarMapValue::Value(v) => *v,
        })
    }
}
