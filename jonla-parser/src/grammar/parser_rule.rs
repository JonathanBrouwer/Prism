use crate::core::adaptive::{GrammarState, RuleState};
use crate::core::cache::PCache;
use crate::core::context::{ParserContext, PR, RawEnv};
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::parser_rule_body::parser_body_cache_recurse;
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::Arc;
use crate::grammar::grammar::Action;

pub fn parser_rule<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone, A: Action<'grm>>(
    rules: &'b GrammarState<'b, 'grm, A>,
    rule: &'grm str,
    args: &'a Vec<Arc<RawEnv<'b, 'grm, A>>>,
) -> impl Parser<'b, 'grm, PR<'b, 'grm, A>, E, A> + 'a {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E, A>, context: &ParserContext| {
        let rule_state: &'b RuleState<'b, 'grm, A> =
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
            context,
        );
        res.add_label_implicit(ErrorLabel::Debug(stream.span_to(res.end_pos()), rule));
        res.map(|pr| pr.fresh())
    }
}
