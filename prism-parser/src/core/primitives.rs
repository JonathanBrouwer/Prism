use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::span::Span;
use crate::core::state::ParserState;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;

impl<'arn, Env, E: ParseError<L = ErrorLabel<'arn>>> ParserState<'arn, Env, E> {
    pub fn parse_char(&mut self, f: impl Fn(&char) -> bool, pos: Pos) -> PResult<(Span, char), E> {
        match pos.next(&self.input) {
            // We can parse the character
            (pos_new, Some((span, e))) if f(&e) => PResult::new_ok((span, e), pos, pos_new),
            // Error
            (_, _) => PResult::new_err(E::new(pos), pos),
        }
    }

    pub fn parse_end(&mut self, pos: Pos) -> PResult<(), E> {
        match pos.next(&self.input) {
            (_, Some(_)) => PResult::new_err(E::new(pos), pos),
            (s, None) => PResult::new_empty((), s),
        }
    }
}
