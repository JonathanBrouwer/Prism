use crate::core::context::{PR, PV, ParserContext};
use crate::core::input_table::InputTableIndex;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
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
                        // Update error
                        last_err.set_end(err_pos);

                        // Prepare for next round
                        let skip = ctx.recovery_points.get_mut(&err_pos).unwrap();
                        let (next_skip, opt) = skip.next(&self.input);
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

    pub fn recovery_point(&mut self) -> PResult<PR, E> {}
}
