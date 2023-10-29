use crate::core::adaptive::{GrammarState, RuleId, RuleState};
use crate::core::cache::PCache;
use crate::core::context::{ParserContext, ValWithEnv, PR};
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::parser::parser_rule_body::parser_body_cache_recurse;
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::Arc;

pub fn parser_rule<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
    rules: &'b GrammarState<'b, 'grm>,
    rule: RuleId,
    args: &'a Vec<Arc<ValWithEnv<'b, 'grm>>>,
) -> impl Parser<'b, 'grm, PR<'b, 'grm>, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E>, context: &ParserContext| {
        let rule_state: &'b RuleState<'b, 'grm> = rules
            .get(rule)
            .unwrap_or_else(|| panic!("Rule not found: {rule}"));

        let rule_args = rule_state
            .arg_names
            .iter()
            .cloned()
            .zip_eq(args.iter().cloned())
            .collect::<HashMap<_, _>>();

        let mut res = parser_body_cache_recurse(rules, &rule_state.blocks, &rule_args)
            .parse(stream, cache, context);
        res.add_label_implicit(ErrorLabel::Debug(
            stream.span_to(res.end_pos()),
            rule_state.name,
        ));
        res.map(|pr| pr.fresh())
    }
}
