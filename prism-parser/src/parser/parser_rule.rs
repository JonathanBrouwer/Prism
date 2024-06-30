use crate::core::adaptive::{GrammarState, RuleId, RuleState};
use crate::core::context::ParserContext;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::state::PState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::parser::parser_rule_body::parser_body_cache_recurse;
use crate::parser::var_map::{VarMap, VarMapValue};
use crate::rule_action::action_result::ActionResult;
use by_address::ByAddress;
use itertools::Itertools;

pub fn parser_rule<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
    rules: &'arn GrammarState<'arn, 'grm>,
    rule: RuleId,
    args: &'a [VarMapValue<'arn, 'grm>],
) -> impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E> + 'a {
    move |pos: Pos, state: &mut PState<'arn, 'grm, E>, context: ParserContext| {
        let rule_state: &'arn RuleState<'arn, 'grm> = rules
            .get(rule)
            .unwrap_or_else(|| panic!("Rule not found: {rule}"));

        let rule_args = VarMap::from_iter(
            rule_state
                .arg_names
                .iter()
                .cloned()
                .zip_eq(args.iter().cloned()),
            state.alloc.alo_varmap,
        );

        let mut res = parser_body_cache_recurse(rules, (ByAddress(&rule_state.blocks), rule_args))
            .parse(pos, state, context);
        res.add_label_implicit(ErrorLabel::Debug(
            pos.span_to(res.end_pos()),
            rule_state.name,
        ));
        res.map(|pr| pr)
    }
}
