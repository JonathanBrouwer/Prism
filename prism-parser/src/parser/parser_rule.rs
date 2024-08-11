use crate::core::adaptive::{GrammarState, RuleId, RuleState};
use crate::core::context::ParserContext;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::state::ParserState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::action_result::ActionResult;
use crate::parser::parser_rule_body::parser_body_cache_recurse;
use crate::parser::var_map::{VarMap, VarMapValue};

pub fn parser_rule<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
    rules: &'arn GrammarState<'arn, 'grm>,
    rule: RuleId,
    args: &'a [VarMapValue<'arn, 'grm>],
) -> impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E> + 'a {
    move |pos: Pos, state: &mut ParserState<'arn, 'grm, E>, context: ParserContext| {
        let rule_state: &'arn RuleState<'arn, 'grm> = rules
            .get(rule)
            .unwrap_or_else(|| panic!("Rule not found: {rule}"));

        assert_eq!(rule_state.args.len(), args.len());
        let rule_args = VarMap::from_iter(
            rule_state.args.iter().cloned().zip(args.iter().cloned()),
            state.alloc,
        );

        let mut res = parser_body_cache_recurse(rules, (rule_state.blocks, rule_args))
            .parse(pos, state, context);
        res.add_label_implicit(ErrorLabel::Debug(
            pos.span_to(res.end_pos()),
            rule_state.name,
        ));
        res.map(|pr| pr)
    }
}
