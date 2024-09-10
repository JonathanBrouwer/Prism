use crate::core::pos::Pos;
use crate::error::aggregate_error::AggregatedParseError;
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

    pub fn parse_chars(&mut self, chars: impl Iterator<Item = char>) -> PResult<E> {
        for expect in chars {
            self.parse_char(|got| expect == *got)?;
        }
        PResult::Ok(())
    }

    pub fn parse_eof(&mut self) -> Result<(), AggregatedParseError<'grm, E>> {
        if self.sequence_state.pos.next(self.input).1.is_some() {
            self.add_error(E::new(
                self.sequence_state.pos.span_to(Pos::end(self.input)),
            ));
            return Err(self.completely_fail());
        }
        Ok(())
    }
}
