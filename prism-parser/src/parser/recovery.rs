use crate::core::context::{PV, ParserContext};
use crate::core::input_table::InputTableIndex;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use crate::parsable::parsed::ArcExt;
use crate::parsable::void::Void;
use std::sync::Arc;

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
                Err(mut next_err) => {
                    // // Has the error stayed the same?
                    // if let Some(last_err) = errors.last_mut() && last_err.span().start_pos() == next_err.span().start_pos() {
                    //     ctx.recovery_points.insert()
                    //
                    //     // last_err.set_end(next_err.span().end_pos());
                    // } else {
                    //
                    // }

                    // https://github.com/JonathanBrouwer/Prism/blob/73f5e7d550583ae449e94be3800da7ef8378ad16/prism-parser/src/core/recovery.rs#L14

                    let end = self.input.end_of_file(next_err.span().start_pos().file());
                    next_err.set_end(end);
                    errors.push(next_err);
                    return (PV::new_multi(Arc::new(Void).to_parsed(), vec![]), errors);
                }
            }
        }
    }
}
