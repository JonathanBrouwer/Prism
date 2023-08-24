use crate::core::adaptive::{GrammarState, RuleState};
use crate::core::cache::PCache;
use crate::core::context::{Ignore, ParserContext, PR};
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::action_result::ActionResult;
use crate::grammar::parser_rule_body::parser_body_cache_recurse;
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::Arc;

pub fn parser_rule<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'b, 'grm>,
    rule: &'grm str,
    args: &'a Vec<Arc<ActionResult<'grm>>>,
) -> impl Parser<'b, 'grm, PR<'grm>, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E>, context: &ParserContext<'b, 'grm>| {
        let rule_state: &'b RuleState<'b, 'grm> =
            rules.get(rule).expect(&format!("Rule not found: {rule}"));

        let args = rule_state
            .args
            .iter()
            .cloned()
            .zip_eq(args.iter().cloned())
            .collect::<HashMap<_, _>>();

        let mut res = parser_body_cache_recurse(rules, &rule_state.blocks, &args).parse(
            stream,
            cache,
            &ParserContext {
                prec_climb_this: Ignore(None),
                prec_climb_next: Ignore(None),
                ..context.clone()
            },
        );
        res.add_label_implicit(ErrorLabel::Debug(stream.span_to(res.end_pos()), rule));
        res.map(|PR(_, v)| PR(HashMap::new(), v))
    }
}
