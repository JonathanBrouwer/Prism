use crate::core::adaptive::{GrammarState, RuleId, RuleState};
use crate::core::arc_ref::BorrowedArcSlice;
use crate::core::context::{PV, ParserContext};
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::error::ParseError;
use crate::error::error_label::ErrorLabel;
use crate::parsable::parsed::Parsed;
use crate::parser::VarMap;
use prism_input::pos::Pos;

impl<Db, E: ParseError<L = ErrorLabel>> ParserState<Db, E> {
    pub fn parse_rule(
        &mut self,
        rules: &GrammarState,
        rule: RuleId,
        args: &[Parsed],
        pos: Pos,
        context: &ParserContext,
        penv: &mut Db,
        eval_ctx: &Parsed,
    ) -> PResult<PV, E> {
        let rule_state: &RuleState = rules
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
                .map(|(_, arg_name)| arg_name.as_str(&self.input).to_string())
                .zip(args.iter().cloned()),
        );

        self.parse_rule_block(
            rules,
            BorrowedArcSlice::new(&rule_state.blocks),
            &rule_args,
            pos,
            context,
            penv,
            eval_ctx,
        )
    }
}
