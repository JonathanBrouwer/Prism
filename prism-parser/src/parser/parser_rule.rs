use crate::core::adaptive::{GrammarState, RuleId, RuleState};
use crate::core::arc_ref::BorrowedArcSlice;
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use crate::parsable::parsed::Parsed;
use crate::parser::VarMap;

impl<Db, E: ParseError<L = ErrorLabel>> ParserState<Db, E> {
    pub fn parse_rule(
        &mut self,
        rules: &GrammarState,
        rule: RuleId,
        args: &[Parsed],
        pos: Pos,
        context: ParserContext,
        penv: &mut Db,
        eval_ctx: &Parsed,
    ) -> PResult<Parsed, E> {
        let rule_state: &RuleState = rules
            .get(rule)
            .unwrap_or_else(|| panic!("Rule not found: {rule}"));

        assert_eq!(
            rule_state.args.len(),
            args.len(),
            "Invalid arguments to rule {}, expected {}, got {}",
            rule_state.name.as_str(),
            rule_state.args.len(),
            args.len()
        );
        let rule_args = VarMap::from_iter(
            rule_state
                .args
                .iter()
                .map(|(_arg_type, arg_name)| arg_name.clone())
                .zip(args.iter().cloned()),
        );

        let mut res = self.parse_rule_block(
            rules,
            BorrowedArcSlice::new(&rule_state.blocks),
            &rule_args,
            pos,
            context,
            penv,
            eval_ctx,
        );
        res.add_label_implicit(ErrorLabel::Debug(
            pos.span_to(res.end_pos()),
            rule_state.name.as_str().to_string(),
        ));
        res.map(|pr| pr)
    }
}
