use crate::parser_core::adaptive::{BlockState, GrammarState};
use crate::parser_core::error::ParseError;
use crate::parser_core::parser::Parser;
use crate::parser_core::parser_cache::ParserCache;
use crate::parser_core::presult::PResult;
use crate::parser_core::stream::StringStream;
use crate::parser_sugar::action_result::ActionResult;
use crate::parser_sugar::error_printer::ErrorLabel;
use crate::parser_sugar::parser_rule_body::parser_body_cache_recurse;
use by_address::ByAddress;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub type PR<'grm> = (
    HashMap<&'grm str, Arc<ActionResult<'grm>>>,
    Arc<ActionResult<'grm>>,
);

pub type PState<'b, 'grm, E> = ParserCache<'grm, 'b, PResult<'grm, PR<'grm>, E>>;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ParserContext<'b, 'grm> {
    pub(crate) recovery_disabled: bool,
    pub(crate) layout_disabled: bool,
    pub(crate) prec_climb_this: Option<ByAddress<&'b [BlockState<'b, 'grm>]>>,
    pub(crate) prec_climb_next: Option<ByAddress<&'b [BlockState<'b, 'grm>]>>,
    pub(crate) recovery_points: Ignore<Arc<HashMap<usize, usize>>>,
}

impl ParserContext<'_, '_> {
    pub fn new() -> Self {
        Self {
            recovery_disabled: false,
            layout_disabled: false,
            prec_climb_this: None,
            prec_climb_next: None,
            recovery_points: Ignore(Arc::new(HashMap::new())),
        }
    }
}

#[derive(Clone, Eq)]
pub struct Ignore<T>(pub T);

impl<T> Hash for Ignore<T> {
    fn hash<H: Hasher>(&self, _: &mut H) {}
}

impl<T> PartialEq for Ignore<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

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

pub fn parser_rule<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'b, 'grm>,
    rule: &'grm str,
) -> impl Parser<'b, 'grm, PR<'grm>, E, PState<'b, 'grm, E>> + 'a {
    move |stream: StringStream<'grm>,
          cache: &mut PState<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>| {
        let body: &'b Vec<BlockState<'b, 'grm>> =
            rules.get(rule).expect(&format!("Rule not found: {rule}"));
        let mut res = parser_body_cache_recurse(rules, body).parse(
            stream,
            cache,
            &ParserContext {
                prec_climb_this: None,
                prec_climb_next: None,
                ..context.clone()
            },
        );
        res.add_label_implicit(ErrorLabel::Debug(stream.span_to(res.get_stream()), rule));
        res.map(|(_, v)| (HashMap::new(), v))
    }
}
