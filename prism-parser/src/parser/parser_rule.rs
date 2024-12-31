use crate::core::adaptive::{GrammarState, RuleId, RuleState};
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::parsable::parsed::Parsed;
use crate::parser::var_map::VarMap;

impl<'arn, 'grm: 'arn, Env: Copy, E: ParseError<L = ErrorLabel<'grm>>>
    ParserState<'arn, 'grm, Env, E>
{
    pub fn parse_rule(
        &mut self,
        rules: &'arn GrammarState<'arn, 'grm>,
        rule: RuleId,
        args: &[Parsed<'arn, 'grm>],
        pos: Pos,
        context: ParserContext,
    ) -> PResult<Parsed<'arn, 'grm>, E> {
        let rule_state: &'arn RuleState<'arn, 'grm> = rules
            .get(rule)
            .unwrap_or_else(|| panic!("Rule not found: {rule}"));

        assert_eq!(
            rule_state.args.len(),
            args.len(),
            "Invalid arguments to rule {}, expected {}, got {}",
            rule_state.name,
            rule_state.args.len(),
            args.len()
        );
        let rule_args = VarMap::from_iter(
            rule_state
                .args
                .iter()
                .map(|(_arg_type, arg_name)| *arg_name)
                .zip(args.iter().cloned()),
            self.alloc,
        );

        let mut res = self.parse_rule_block(rules, rule_state.blocks, rule_args, pos, context);
        res.add_label_implicit(ErrorLabel::Debug(
            pos.span_to(res.end_pos()),
            rule_state.name,
        ));
        res.map(|pr| pr)
    }
}
