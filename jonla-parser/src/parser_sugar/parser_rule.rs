use crate::parser_core::error::ParseError;
use crate::parser_core::parser::Parser;
use crate::parser_core::parser_cache::ParserCache;
use crate::parser_core::presult::PResult;
use crate::parser_sugar::action_result::ActionResult;
use crate::parser_sugar::error_printer::ErrorLabel;
use crate::parser_core::adaptive::{BlockState, GrammarState};
use crate::parser_core::stream::StringStream;
use crate::parser_sugar::parser_rule_body::parser_body_cache_recurse;
use by_address::ByAddress;
use std::collections::HashMap;
use std::sync::Arc;

pub type PR<'grm> = (
    HashMap<&'grm str, Arc<ActionResult<'grm>>>,
    Arc<ActionResult<'grm>>,
);

pub type PState<'b, 'grm, E> = ParserCache<'grm, 'b, PResult<'grm, PR<'grm>, E>>;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ParserContext<'b, 'grm> {
    pub(crate) layout_disabled: bool,
    pub(crate) prec_climb_this: Option<ByAddress<&'b [BlockState<'b, 'grm>]>>,
    pub(crate) prec_climb_next: Option<ByAddress<&'b [BlockState<'b, 'grm>]>>,
}

impl ParserContext<'_, '_> {
    pub fn new() -> Self {
        Self {
            layout_disabled: false,
            prec_climb_this: None,
            prec_climb_next: None,
        }
    }
}

pub fn parser_rule<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'b, 'grm>,
    rule: &'grm str,
    context: &'a ParserContext<'b, 'grm>,
) -> impl Parser<'grm, PR<'grm>, E, PState<'b, 'grm, E>> + 'a {
    move |stream: StringStream<'grm>, cache: &mut PState<'b, 'grm, E>| {
        let body: &'b Vec<BlockState<'b, 'grm>> =
            rules.get(rule).expect(&format!("Rule not found: {rule}"));
        let mut res = parser_body_cache_recurse(
            rules,
            body,
            &ParserContext {
                prec_climb_this: None,
                prec_climb_next: None,
                ..*context
            },
        )
        .parse(stream, cache);
        res.add_label_implicit(ErrorLabel::Debug(stream.span_to(res.get_stream()), rule));
        res.map(|(_, v)| (HashMap::new(), v))
    }
}
