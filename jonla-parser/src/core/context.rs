use crate::core::adaptive::BlockState;
use crate::core::pos::Pos;
use crate::rule_action::{RuleAction};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use crate::core::span::Span;

#[derive(Clone, Debug)]
pub struct PR<'b, 'grm>(
    pub HashMap<&'grm str, Arc<PR<'b, 'grm>>>,
    pub Raw<'b, 'grm>
);

#[derive(Clone, Debug)]
pub enum Raw<'b, 'grm> {
    Internal(&'static str),
    Value(Span),
    Action(&'b RuleAction<'grm>),
    List(Span, Vec<PR<'b, 'grm>>),
    Rule(&'grm str),
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ParserContext<'b, 'grm> {
    pub(crate) recovery_disabled: bool,
    pub(crate) layout_disabled: bool,
    pub(crate) prec_climb_this: Ignore<Option<&'b [BlockState<'b, 'grm>]>>,
    pub(crate) prec_climb_next: Ignore<Option<&'b [BlockState<'b, 'grm>]>>,
    pub(crate) recovery_points: Ignore<Arc<HashMap<Pos, Pos>>>,
}

impl ParserContext<'_, '_> {
    pub fn new() -> Self {
        Self {
            recovery_disabled: false,
            layout_disabled: false,
            prec_climb_this: Ignore(None),
            prec_climb_next: Ignore(None),
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
