use std::os::linux::raw::stat;
use crate::core::context::ParserContext;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::span::Span;
use crate::core::state::ParserState;
use crate::error::error_printer::ErrorLabel;
use crate::error::error_printer::ErrorLabel::Debug;
use crate::error::ParseError;

impl<'arn, 'grm, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, E> {
    #[inline(always)]
    pub fn parse_char(
        &mut self,
        f: impl Fn(&char) -> bool,
        pos: Pos,
    ) -> PResult<(Span, char), E> {
        match pos.next(self.input) {
            // We can parse the character
            (pos_new, Some((span, e))) if f(&e) => PResult::new_ok((span, e), pos, pos_new),
            // Error
            (pos_new, _) => PResult::new_err(E::new(pos.span_to(pos_new)), pos),
        }
    }

    #[inline(always)]
    pub fn parse_end(&mut self, pos: Pos) -> PResult<(), E> {
        match pos.next(self.input) {
            (s, Some(_)) => PResult::new_err(E::new(pos.span_to(s)), pos),
            (s, None) => PResult::new_empty((), s),
        }
    }
}

#[inline(always)]
pub fn positive_lookahead<'arn, 'grm: 'arn, O, E: ParseError<L = ErrorLabel<'grm>>>(
    p: &impl Parser<'arn, 'grm, O, E>,
) -> impl Parser<'arn, 'grm, O, E> + '_ {
    move |pos: Pos,
          state: &mut ParserState<'arn, 'grm, E>,
          context: ParserContext|
          -> PResult<O, E> {
        match p.parse(pos, state, context) {
            POk(o, _, _, err) => POk(o, pos, pos, err),
            PErr(e, s) => PErr(e, s),
        }
    }
}

#[inline(always)]
pub fn negative_lookahead<'arn, 'grm: 'arn, O, E: ParseError<L = ErrorLabel<'grm>>>(
    p: &impl Parser<'arn, 'grm, O, E>,
) -> impl Parser<'arn, 'grm, (), E> + '_ {
    move |pos: Pos,
          state: &mut ParserState<'arn, 'grm, E>,
          context: ParserContext|
          -> PResult<(), E> {
        match p.parse(pos, state, context) {
            POk(_, _, _, _) => PResult::new_err(E::new(pos.span_to(pos)), pos),
            PErr(_, _) => PResult::new_empty((), pos),
        }
    }
}
