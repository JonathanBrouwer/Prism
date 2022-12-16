use std::collections::HashMap;
use std::sync::Arc;
use std::ops::{Deref, DerefMut};
use std::hash::{Hash, Hasher};
use crate::parser_core::adaptive::BlockState;
use crate::parser_core::parser_cache::ParserCache;
use crate::parser_core::presult::PResult;
use crate::parser_sugar::action_result::ActionResult;

pub type PR<'grm> = (
    HashMap<&'grm str, Arc<ActionResult<'grm>>>,
    Arc<ActionResult<'grm>>,
);

pub type PState<'b, 'grm, E> = ParserCache<'grm, 'b, PResult<'grm, PR<'grm>, E>>;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ParserContext<'b, 'grm> {
    pub(crate) recovery_disabled: Option<usize>,
    pub(crate) layout_disabled: bool,
    pub(crate) prec_climb_this: Ignore<Option<&'b [BlockState<'b, 'grm>]>>,
    pub(crate) prec_climb_next: Ignore<Option<&'b [BlockState<'b, 'grm>]>>,
    pub(crate) recovery_points: Ignore<Arc<HashMap<usize, usize>>>,
}

impl ParserContext<'_, '_> {
    pub fn new() -> Self {
        Self {
            recovery_disabled: None,
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
