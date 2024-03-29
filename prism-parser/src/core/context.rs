use crate::core::cow::Cow;
use crate::core::pos::Pos;
use crate::rule_action::action_result::ActionResult;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct PR<'arn, 'grm> {
    pub free: HashMap<&'grm str, Cow<'arn, ActionResult<'arn, 'grm>>>,
    pub rtrn: Cow<'arn, ActionResult<'arn, 'grm>>,
}

impl<'arn, 'grm> PR<'arn, 'grm> {
    pub fn with_cow_rtrn(rtrn: Cow<'arn, ActionResult<'arn, 'grm>>) -> Self {
        Self {
            free: HashMap::new(),
            rtrn,
        }
    }

    pub fn with_rtrn(rtrn: ActionResult<'arn, 'grm>) -> Self {
        Self {
            free: HashMap::new(),
            rtrn: Cow::Owned(rtrn),
        }
    }
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
