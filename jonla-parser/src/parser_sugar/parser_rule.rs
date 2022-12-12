use crate::parser_core::error::ParseError;
use crate::parser_core::parser::Parser;
use crate::parser_core::parser_state::ParserState;
use crate::parser_core::presult::PResult;
use crate::parser_sugar::action_result::ActionResult;
use crate::parser_sugar::error_printer::ErrorLabel;
use crate::parser_sugar::parser_layout::full_input_layout;

use crate::parser_core::stream::StringStream;
use crate::parser_sugar::parser_rule_body::parser_body_cache_recurse;
use by_address::ByAddress;
use std::collections::HashMap;
use std::rc::Rc;
use crate::grammar::GrammarFile;
use crate::parser_core::adaptive::{BlockState, GrammarState};

pub type PR<'grm> = (
    HashMap<&'grm str, Rc<ActionResult<'grm>>>,
    Rc<ActionResult<'grm>>,
);

pub type PState<'b, 'grm, E> = ParserState<'grm, 'b, PResult<'grm, PR<'grm>, E>>;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct ParserContext<'b, 'grm> {
    pub(crate) layout_disabled: bool,
    pub(crate) prec_climb_this: Option<ByAddress<&'b [BlockState<'grm>]>>,
    pub(crate) prec_climb_next: Option<ByAddress<&'b [BlockState<'grm>]>>,
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

pub fn run_parser_rule<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'grm GrammarFile,
    rule: &'grm str,
    stream: StringStream<'grm>,
) -> Result<PR<'grm>, E> {
    let context = ParserContext::new();
    let mut state = ParserState::new();
    let grammar_state = GrammarState::new(&rules);

    let x = full_input_layout(&grammar_state, &parser_rule(&grammar_state, rule, &context), &context)
        .parse(stream, &mut state)
        .collapse();
    x
}

pub fn parser_rule<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'grm>,
    rule: &'grm str,
    context: &'a ParserContext<'b, 'grm>,
) -> impl Parser<'grm, PR<'grm>, E, PState<'b, 'grm, E>> + 'a {
    move |stream: StringStream<'grm>, state: &mut PState<'b, 'grm, E>| {
        let body: &'b Vec<BlockState<'grm>> = rules
            .rules
            .get(rule)
            .expect(&format!("Rule not found: {rule}"));
        let mut res = parser_body_cache_recurse(
            rules,
            body,
            &ParserContext {
                prec_climb_this: None,
                prec_climb_next: None,
                ..*context
            },
        ).parse(stream, state);
        res.add_label_implicit(ErrorLabel::Debug(stream.span_to(res.get_stream()), rule));
        res
    }
}
