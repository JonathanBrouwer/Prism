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
pub fn repeat_delim<'arn, 'grm: 'arn, OP, OD, E: ParseError<L = ErrorLabel<'grm>>>(
    item: impl Parser<'arn, 'grm, OP, E>,
    delimiter: impl Parser<'arn, 'grm, OD, E>,
    min: usize,
    max: Option<usize>,
) -> impl Parser<'arn, 'grm, Vec<OP>, E> {
    move |pos: Pos,
          state: &mut ParserState<'arn, 'grm, E>,
          context: ParserContext|
          -> PResult<Vec<OP>, E> {
        let mut last_res: PResult<Vec<OP>, E> = PResult::new_empty(vec![], pos);

        for i in 0..max.unwrap_or(usize::MAX) {
            let pos = last_res.end_pos();
            let part = if i == 0 {
                item.parse(pos, state, context)
            } else {
                delimiter.parse(pos, state, context).merge_seq_parser(&item, state, context).map(|x| x.1)
            };
            let should_continue = part.is_ok();

            if i < min {
                last_res = last_res.merge_seq(part).map(|(mut vec, item)| {
                    vec.push(item);
                    vec
                });
            } else {
                last_res = last_res.merge_seq_opt(part).map(|(mut vec, item)| {
                    if let Some(item) = item {
                        vec.push(item);
                    }
                    vec
                });
            };

            if !should_continue {
                break;
            };

            // If the result is OK and the last pos has not changed, we got into an infinite loop
            // We break out with an infinite loop error
            // The i != 0 check is to make sure to take the delim into account
            if i != 0 && last_res.end_pos() <= pos {
                let span = pos.span_to(pos);
                let mut e = E::new(span);
                e.add_label_explicit(Debug(span, "INFLOOP"));
                return PResult::new_err(e, pos);
            }
        }

        last_res
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
