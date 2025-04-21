use crate::core::adaptive::{GrammarState, RuleId};
use crate::core::context::{PV, ParserContext};
use crate::core::input::Input;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::state::ParserState;
use crate::core::tokens::{Token, TokenType, Tokens};
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use crate::parsable::parsed::ArcExt;
use crate::parsable::void::Void;
use crate::parser::VarMap;
use std::sync::Arc;

impl<Db, E: ParseError<L = ErrorLabel>> ParserState<Db, E> {
    pub fn parse_with_layout(
        &mut self,
        rules: &GrammarState,
        vars: &VarMap,
        sub: impl Fn(&mut ParserState<Db, E>, Pos, &mut Db) -> PResult<PV, E>,
        pos: Pos,
        context: &ParserContext,
        penv: &mut Db,
    ) -> PResult<PV, E> {
        if context.layout_disabled {
            return sub(self, pos, penv);
        }
        let Some(layout) = vars.get(&Input::from_const("layout")) else {
            return sub(self, pos, penv);
        };
        let layout = *layout.value_ref::<RuleId>();

        let mut res = PResult::new_empty(Vec::new(), pos);
        loop {
            let new_res = sub(self, res.end_pos(), penv);
            if new_res.is_ok() {
                return res.merge_seq(new_res).map(|(mut tokens, o)| {
                    tokens.push(o.tokens);
                    PV::new_multi(o.parsed, tokens)
                });
            }

            let pos_before_layout = new_res.end_pos();
            // Add in optional error information from sub_res, then require another layout token
            let new_res = res.merge_seq_opt(new_res).merge_seq_chain(|pos| {
                self.parse_rule(
                    rules,
                    layout,
                    &[],
                    pos,
                    &ParserContext {
                        layout_disabled: true,
                        ..context.clone()
                    },
                    penv,
                    &Arc::new(Void).to_parsed(),
                )
            });
            match new_res {
                // We have parsed more layout, we can try again
                POk {
                    obj: ((mut old, _), _new),
                    start: _,
                    end: new_end_pos,
                    best_err: new_err,
                } if pos_before_layout < new_res.end_pos() => {
                    old.push(Arc::new(Tokens::Single(Token {
                        token_type: TokenType::Layout,
                        span: pos_before_layout.span_to(new_end_pos),
                    })));
                    res = POk {
                        obj: old,
                        start: new_end_pos,
                        end: new_end_pos,
                        best_err: new_err,
                    };
                }
                // We have not parsed more layout ...
                // ... because layout parser did not parse more characters
                POk { .. } => {
                    assert_eq!(pos_before_layout, new_res.end_pos());
                    unreachable!("Cannot happen")
                }
                // ... because layout parser failed
                PErr { err: e, end: pos } => return PErr { err: e, end: pos },
            }
        }
    }

    pub fn parse_end_with_layout(
        &mut self,
        rules: &GrammarState,
        vars: &VarMap,
        pos: Pos,
        context: &ParserContext,
        penv: &mut Db,
    ) -> PResult<PV, E> {
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
