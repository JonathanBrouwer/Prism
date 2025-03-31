use crate::core::adaptive::{GrammarState, RuleId};
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::state::ParserState;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use crate::grammar::identifier::Identifier;
use crate::parsable::parsed::ArcExt;
use crate::parsable::void::Void;
use crate::parser::VarMap;
use std::sync::Arc;

impl<Env, E: ParseError<L = ErrorLabel>> ParserState<Env, E> {
    pub fn parse_with_layout<O>(
        &mut self,
        rules: &GrammarState,
        vars: &VarMap,
        sub: impl Fn(&mut ParserState<Env, E>, Pos, &mut Env) -> PResult<O, E>,
        pos: Pos,
        context: ParserContext,
        penv: &mut Env,
    ) -> PResult<O, E> {
        if context.layout_disabled {
            return sub(self, pos, penv);
        }
        let Some(layout) = vars.get_ident(Identifier::from_const("layout"), &self.input) else {
            return sub(self, pos, penv);
        };
        let layout = *layout.value_ref::<RuleId>();

        let mut res = PResult::new_empty((), pos);
        loop {
            let new_res = sub(self, res.end_pos(), penv);
            if new_res.is_ok() {
                return res.merge_seq(new_res).map(|(_, o)| o);
            }

            let pos_before_layout = new_res.end_pos();
            // Add in optional error information from sub_res, then require another layout token
            let new_res = res.merge_seq_opt(new_res).merge_seq_chain(|pos| {
                self.parse_rule(
                    rules,
                    layout,
                    &[],
                    pos,
                    ParserContext {
                        layout_disabled: true,
                        ..context
                    },
                    penv,
                    &Arc::new(Void).to_parsed(),
                )
            });
            match new_res {
                // We have parsed more layout, we can try again
                POk(_, _, new_end_pos, new_err) if pos_before_layout < new_res.end_pos() => {
                    res = POk((), new_end_pos, new_end_pos, new_err);
                }
                // We have not parsed more layout ...
                // ... because layout parser did not parse more characters
                POk(_, _, _, err) => {
                    let (e, pos) = err.unwrap();
                    return PErr(e, pos);
                }
                // ... because layout parser failed
                PErr(e, pos) => return PErr(e, pos),
            }
        }
    }

    pub fn parse_end_with_layout(
        &mut self,
        rules: &GrammarState,
        vars: &VarMap,
        pos: Pos,
        context: ParserContext,
        penv: &mut Env,
    ) -> PResult<(), E> {
        self.parse_with_layout(
            rules,
            vars,
            |state, pos, _penv| state.parse_end(pos),
            pos,
            context,
            penv,
        )
    }
}
