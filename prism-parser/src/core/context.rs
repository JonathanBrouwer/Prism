use crate::core::adaptive::RuleId;
use crate::core::pos::Pos;
use crate::core::span::Span;
use crate::rule_action::RuleAction;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct PR<'b, 'grm> {
    pub free: HashMap<&'grm str, Arc<ValWithEnv<'b, 'grm>>>,
    pub rtrn: ValWithEnv<'b, 'grm>,
}

impl<'b, 'grm> PR<'b, 'grm> {
    pub fn from_raw(rtrn: Val<'b, 'grm>) -> Self {
        Self {
            free: HashMap::new(),
            rtrn: ValWithEnv::from_raw(rtrn),
        }
    }

    /// Returns self with fresh free variables
    pub fn fresh(self) -> Self {
        PR {
            free: HashMap::new(),
            rtrn: self.rtrn,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ValWithEnv<'b, 'grm> {
    pub env: HashMap<&'grm str, Arc<ValWithEnv<'b, 'grm>>>,
    pub value: Val<'b, 'grm>,
}

impl<'b, 'grm> ValWithEnv<'b, 'grm> {
    pub fn from_raw(value: Val<'b, 'grm>) -> Self {
        Self {
            env: HashMap::new(),
            value,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Val<'b, 'grm> {
    Void,
    Text(Span),
    Action(&'b RuleAction<'grm>),
    List(Span, Vec<ValWithEnv<'b, 'grm>>),
    Rule(RuleId),
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ParserContext {
    pub(crate) recovery_disabled: bool,
    pub(crate) layout_disabled: bool,
    pub(crate) recovery_points: Ignore<Arc<HashMap<Pos, Pos>>>,
}

impl Default for ParserContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserContext {
    pub fn new() -> Self {
        Self {
            recovery_disabled: false,
            layout_disabled: false,
            recovery_points: Ignore(Arc::new(HashMap::new())),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Ignore<T>(pub T);

impl<T> Hash for Ignore<T> {
    fn hash<H: Hasher>(&self, _: &mut H) {}
}

impl<T> PartialEq for Ignore<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T> Eq for Ignore<T> {}

impl<T> Deref for Ignore<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Ignore<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}