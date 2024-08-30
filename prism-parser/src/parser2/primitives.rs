use crate::core::pos::Pos;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::parser2::PResult;

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>>
    crate::parser2::ParserState<'arn, 'grm, E>
{
    pub fn parse_char(&mut self, f: impl Fn(&char) -> bool) {
        todo!()
        // match self.seq_state.pos.next(self.input) {
        //     // We can parse the character
        //     (pos_new, Some((span, e))) if f(&e) => {
        //         self.seq_state.pos = pos_new;
        //     }
        //     // Error
        //     (pos_new, _) => {
        //         self.fail(E::new(self.seq_state.pos.span_to(pos_new)))
        //     },
        // }
    }
}
