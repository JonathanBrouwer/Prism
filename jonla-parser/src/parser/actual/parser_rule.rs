use crate::parser::actual::action_result::ActionResult;
use crate::parser::actual::error_printer::ErrorLabel;
use crate::parser::actual::parser_layout::full_input_layout;
use crate::parser::core::error::ParseError;
use crate::parser::core::parser::Parser;
use crate::parser::core::parser_state::ParserState;
use crate::parser::core::presult::PResult;

use crate::parser::core::stream::Stream;

use crate::grammar::{Block, Blocks, GrammarFile};
use crate::parser::actual::parser_rule_body::parser_body_cache_recurse;
use by_address::ByAddress;
use std::collections::HashMap;
use std::rc::Rc;

pub type PR<'grm> = (
    HashMap<&'grm str, Rc<ActionResult<'grm>>>,
    Rc<ActionResult<'grm>>,
);

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ParserContext<'grm> {
    pub(crate) layout_disabled: bool,
    pub(crate) prec_climb_this: Option<ByAddress<&'grm [Block]>>,
    pub(crate) prec_climb_next: Option<ByAddress<&'grm [Block]>>,
}

impl ParserContext<'_> {
    pub fn new() -> Self {
        Self {
            layout_disabled: false,
            prec_climb_this: None,
            prec_climb_next: None,
        }
    }
}

pub fn run_parser_rule<'b, 'grm: 'b, S: Stream, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'grm GrammarFile,
    rule: &'grm str,
    stream: S,
) -> Result<PR<'grm>, E> {
    let context = ParserContext::new();
    let mut state = ParserState::new();
    let x = full_input_layout(rules, &parser_rule(rules, rule, &context), &context)
        .parse(stream, &mut state)
        .collapse();
    x
}

pub fn parser_rule<'a, 'b: 'a, 'grm: 'b, S: Stream, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'grm GrammarFile,
    rule: &'grm str,
    context: &'a ParserContext<'grm>,
) -> impl Parser<PR<'grm>, S, E, ParserState<'b, PResult<PR<'grm>, E, S>>> + 'a {
    move |stream: S,
          state: &mut ParserState<'b, PResult<PR<'grm>, E, S>>|
          -> PResult<PR<'grm>, E, S> {
        let body: &'grm Blocks = &rules
            .rules
            .get(rule)
            .expect(&format!("Rule not found: {rule}"))
            .blocks;
        let mut res = parser_body_cache_recurse(
            rules,
            body,
            &ParserContext {
                prec_climb_this: None,
                prec_climb_next: None,
                ..*context
            },
        )
        .parse(stream, state);
        res.add_label_implicit(ErrorLabel::Debug(stream.span_to(res.get_stream()), rule));
        res
    }
}
