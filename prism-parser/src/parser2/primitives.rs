use crate::core::pos::Pos;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::parser2::PResult;

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>>
    crate::parser2::ParserState<'arn, 'grm, E>
{
    pub fn parse_char(&mut self, f: impl Fn(&char) -> bool) -> PResult<E> {
        todo!()
    }
}

// ) -> impl Parser<'arn, 'grm, (Span, char), E> {
//     move |pos: Pos,
//           state: &mut ParserState<'arn, 'grm, E>,
//           _: ParserContext|
//           -> PResult<(Span, char), E> {
//         match pos.next(state.input) {
//             // We can parse the character
//             (pos_new, Some((span, e))) if f(&e) => PResult::new_ok((span, e), pos, pos_new),
//             // Error
//             (pos_new, _) => PResult::new_err(E::new(pos.span_to(pos_new)), pos),
//         }
//     }
// }
