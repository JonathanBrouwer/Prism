use crate::core::adaptive::{GrammarState, RuleId, RuleState};
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use crate::parsable::parsed::Parsed;
use crate::parser::VarMap;

impl<'arn, Env, E: ParseError<L = ErrorLabel>> ParserState<'arn, Env, E> {
    pub fn parse_rule(
        &mut self,
        rules: &'arn GrammarState<'arn>,
        rule: RuleId,
        args: &[Parsed<'arn>],
        pos: Pos,
        context: ParserContext,
        penv: &mut Env,
        eval_ctx: Parsed<'arn>,
    ) -> PResult<Parsed<'arn>, E> {
        let rule_state: &'arn RuleState<'arn> = rules
            .get(rule)
            .unwrap_or_else(|| panic!("Rule not found: {rule}"));

        assert_eq!(
            rule_state.args.len(),
            args.len(),
            "Invalid arguments to rule {}, expected {}, got {}",
            rule_state.name.as_str(&self.input),
            rule_state.args.len(),
            args.len()
        );
        let rule_args = VarMap::from_iter(
            rule_state
                .args
                .iter()
                .map(|(_arg_type, arg_name)| arg_name.as_str(&self.input))
                .zip(args.iter().cloned()),
            self.alloc,
        );

        let mut res = self.parse_rule_block(
            rules,
            rule_state.blocks,
            rule_args,
            pos,
            context,
            penv,
            eval_ctx,
        );
        res.add_label_implicit(ErrorLabel::Debug(
            pos.span_to(res.end_pos()),
            rule_state.name.as_str(&self.input).to_string(),
        ));
        res.map(|pr| pr)
    }
}
