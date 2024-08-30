use crate::core::context::PR;
use crate::core::pos::Pos;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::parser2::PResult;

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>>
    crate::parser2::ParserState<'arn, 'grm, E>
{
    pub fn parse_char(&mut self, f: impl Fn(&char) -> bool) -> PResult<E> {
        match self.sequence_state.pos.next(self.input) {
            // We can parse the character
            (span, Some(e)) if f(&e) => {
                self.sequence_state.pos = span.end;
                PResult::Ok(())
            }
            // Error
            (span, _) => PResult::Err(E::new(span)),
        }
    }

    pub fn parse_chars(&mut self, mut chars: impl Iterator<Item = char>) -> PResult<E> {
        while let Some(expect) = chars.next() {
            self.parse_char(|got| expect == *got)?;
        }
        PResult::Ok(())
    }
}
