use crate::core::adaptive::GrammarState;
use crate::core::context::{PV, ParserContext};
use crate::core::input_table::InputTableIndex;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::core::tokens::Tokens;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use crate::parsable::Parsable;
use crate::parsable::parsed::ArcExt;
use crate::parsable::void::Void;
use crate::parser::VarMap;
use std::sync::Arc;

impl<Db, E: ParseError<L = ErrorLabel>> ParserState<Db, E> {
    pub fn parse_with_recovery(
        &mut self,
        sub: impl Fn(&mut ParserState<Db, E>, &mut Db) -> PResult<PV, E>,
        file: InputTableIndex,
        penv: &mut Db,
    ) -> (PV, Vec<E>) {
        match sub(self, penv).collapse() {
            Ok(v) => (v, vec![]),
            Err(err) => (PV::new_multi(Arc::new(Void).to_parsed(), vec![]), vec![err]),
        }
    }
}
