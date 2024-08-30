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

#[inline(always)]
pub fn single<'arn, 'grm: 'arn, E: ParseError>(
    f: impl Fn(&char) -> bool,
) -> impl Parser<'arn, 'grm, (Span, char), E> {
    move |pos: Pos,
          state: &mut ParserState<'arn, 'grm, E>,
          _: ParserContext|
          -> PResult<(Span, char), E> {
        match pos.next(state.input) {
            // We can parse the character
            (span, Some(e)) if f(&e) => PResult::new_ok((span, e), span.start, span.end),
            // Error
            (span, _) => PResult::new_err(E::new(span), pos),
        }
    }
}

#[inline(always)]
pub fn seq2<'arn, 'grm: 'arn, 'a, O1, O2, E: ParseError>(
    p1: &'a impl Parser<'arn, 'grm, O1, E>,
    p2: &'a impl Parser<'arn, 'grm, O2, E>,
) -> impl Parser<'arn, 'grm, (O1, O2), E> + 'a {
    move |pos: Pos,
          state: &mut ParserState<'arn, 'grm, E>,
          context: ParserContext|
          -> PResult<(O1, O2), E> {
        let res1 = p1.parse(pos, state, context);
        let end_pos = res1.end_pos();
        res1.merge_seq(p2.parse(end_pos, state, context))
    }
}

#[inline(always)]
pub fn choice2<'arn, 'grm: 'arn, 'a, O, E: ParseError>(
    p1: &'a impl Parser<'arn, 'grm, O, E>,
    p2: &'a impl Parser<'arn, 'grm, O, E>,
) -> impl Parser<'arn, 'grm, O, E> + 'a {
    move |pos: Pos,
          state: &mut ParserState<'arn, 'grm, E>,
          context: ParserContext|
          -> PResult<O, E> {
        p1.parse(pos, state, context)
            .merge_choice(p2.parse(pos, state, context))
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
                seq2(&delimiter, &item)
                    .parse(pos, state, context)
                    .map(|x| x.1)
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
pub fn end<'arn, 'grm: 'arn, E: ParseError>() -> impl Parser<'arn, 'grm, (), E> {
    move |pos: Pos, state: &mut ParserState<'arn, 'grm, E>, _: ParserContext| -> PResult<(), E> {
        match pos.next(state.input) {
            (s, Some(_)) => PResult::new_err(E::new(s), pos),
            (s, None) => PResult::new_empty((), s.start),
        }
    }
}

#[inline(always)]
pub fn positive_lookahead<'arn, 'grm: 'arn, O, E: ParseError>(
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
pub fn negative_lookahead<'arn, 'grm: 'arn, O, E: ParseError>(
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
