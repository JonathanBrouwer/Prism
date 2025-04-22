use crate::core::context::{PV, ParserContext};
use crate::core::input_table::InputTableIndex;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::core::tokens::TokenType;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use crate::parsable::parsed::ArcExt;
use crate::parsable::void::Void;
use std::sync::Arc;

//TODO https://github.com/JonathanBrouwer/Prism/blob/73f5e7d550583ae449e94be3800da7ef8378ad16/prism-parser/src/core/recovery.rs#L14

impl<Db, E: ParseError<L = ErrorLabel>> ParserState<Db, E> {
    pub fn parse_with_recovery(
        &mut self,
        sub: impl Fn(&mut ParserState<Db, E>, &ParserContext, &mut Db) -> PResult<PV, E>,
        _file: InputTableIndex,
        penv: &mut Db,
    ) -> (PV, Vec<E>) {
        // Contains all errors we found, the last error is the one we're currently trying to recover from
        let mut errors = vec![];
        let mut ctx = ParserContext::default();

        loop {
            match sub(self, &ctx, penv).collapse() {
                Ok(pv) => return (pv, errors),
                Err(next_err) => {
                    // Has the error stayed the same?
                    let err_pos = next_err.span().start_pos();
                    if let Some(last_err) = errors.last_mut()
                        && last_err.span().start_pos() == err_pos
                    {
                        // Prepare for next round
                        let skip = ctx.recovery_points.get_mut(&err_pos).unwrap();
                        let (next_skip, opt) = skip.next(&self.input);

                        // Update error
                        last_err.set_end(next_skip);

                        if opt.is_some() {
                            // There is more input available, retry parsing
                            *skip = next_skip;
                            continue;
                        } else {
                            // No more input available
                            //TODO is this reachable?
                            // unreachable!()

                            return (PV::new_multi(Arc::new(Void).to_parsed(), vec![]), errors);
                        }
                    } else {
                        // Making negative progress should not be possible
                        if let Some(last_err) = errors.last() {
                            assert!(err_pos >= last_err.span().start_pos());
                        }

                        errors.push(next_err);
                        ctx.recovery_points.insert(err_pos, err_pos);
                    }
                }
            }
        }
    }

    pub fn parse_with_recovery_point(
        &mut self,
        sub: impl Fn(&mut ParserState<Db, E>, Pos) -> PResult<PV, E>,
        pos: Pos,
        context: &ParserContext,
    ) -> PResult<PV, E> {
        // If recovery is disabled or there is no recovery point, give up
        if context.recovery_disabled {
            return sub(self, pos);
        }
        let Some(&jump_pos) = context.recovery_points.get(&pos) else {
            return sub(self, pos);
        };

        // Try parsing at jump_pos
        let res = sub(self, jump_pos);
        let PResult::PErr { err, end } = res else {
            return res;
        };

        PResult::POk {
            obj: PV::new_single(
                Arc::new(Void).to_parsed(),
                TokenType::Error,
                pos.span_to(jump_pos),
            ),
            start: pos,
            end: jump_pos,
            best_err: Some((err, end)),
        }
    }
}
