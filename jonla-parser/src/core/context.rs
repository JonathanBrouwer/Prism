use crate::core::pos::Pos;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use crate::core::adaptive::RuleId;
use crate::core::span::Span;
use crate::grammar::grammar::{Action, GrammarFile};

#[derive(Clone, Debug)]
pub struct PR<'b, 'grm, A> {
    pub free: HashMap<&'grm str, Arc<RawEnv<'b, 'grm, A>>>,
    pub rtrn: RawEnv<'b, 'grm, A>,
}

impl<'b, 'grm, A: Action<'grm>> PR<'b, 'grm, A> {
    pub fn from_raw(rtrn: Raw<'b, 'grm, A>) -> Self {
        Self {
            free: HashMap::new(),
            rtrn: RawEnv::from_raw(rtrn)
        }
    }

    /// Returns self with fresh free variables
    pub fn fresh(self) -> Self {
        PR { free: HashMap::new(), rtrn: self.rtrn }
    }
}

#[derive(Clone, Debug)]
pub struct RawEnv<'b, 'grm, A> {
    pub env: HashMap<&'grm str, Arc<RawEnv<'b, 'grm, A>>>,
    pub value: Raw<'b, 'grm, A>
}

impl<'b, 'grm, A: Action<'grm>> RawEnv<'b, 'grm, A> {
    pub fn from_raw(value: Raw<'b, 'grm, A>) -> Self {
        Self {
            env: HashMap::new(),
            value
        }
    }
}

#[derive(Clone, Debug)]
pub enum Raw<'b, 'grm, A> {
    Internal(&'static str),
    Value(Span),
    Action(&'b A),
    List(Span, Vec<RawEnv<'b, 'grm, A>>),
    Rule(RuleId),
    Grammar(Arc<GrammarFile<'grm, A>>),
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ParserContext {
    pub(crate) recovery_disabled: bool,
    pub(crate) layout_disabled: bool,
    pub(crate) recovery_points: Ignore<Arc<HashMap<Pos, Pos>>>,
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
