use crate::core::adaptive::GrammarState;
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use crate::parser::VarMap;

impl<Db, E: ParseError<L = ErrorLabel>> ParserState<Db, E> {
    pub fn parse_with_recovery<O>(
        &mut self,
        rules: &GrammarState,
        vars: &VarMap,
        sub: impl Fn(&mut ParserState<Db, E>, Pos, &mut Db) -> PResult<O, E>,
        pos: Pos,
        context: ParserContext,
        penv: &mut Db,
    ) -> PResult<O, E> {
        todo!()
    }
}
