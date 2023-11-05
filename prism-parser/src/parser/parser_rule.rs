use crate::core::adaptive::{GrammarState, RuleId, RuleState};
use crate::core::cache::PCache;
use crate::core::context::{ParserContext, PR};
use crate::core::cow::Cow;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::parser::parser_rule_body::parser_body_cache_recurse;
use crate::rule_action::action_result::ActionResult;
use itertools::Itertools;
use std::collections::HashMap;

pub fn parser_rule<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
    rules: &'arn GrammarState<'arn, 'grm>,
    rule: RuleId,
    args: &'a [Cow<'arn, ActionResult<'arn, 'grm>>],
) -> impl Parser<'arn, 'grm, PR<'arn, 'grm>, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'arn, 'grm, E>, context: &ParserContext| {
        let rule_state: &'arn RuleState<'arn, 'grm> = rules
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
