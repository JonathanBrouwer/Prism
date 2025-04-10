use crate::core::context::{PV, ParserContext};
use crate::core::input_table::InputTableIndex;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use crate::parsable::parsed::ArcExt;
use crate::parsable::void::Void;
use log::error;
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
                Err(mut err) => {
                    let end = self.input.end_of_file(err.span().start_pos().file());
                    err.set_end(end);
                    errors.push(err);
                    return (PV::new_multi(Arc::new(Void).to_parsed(), vec![]), errors);
                }
            }
        }
    }
}
