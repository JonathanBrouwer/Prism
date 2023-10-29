use crate::core::cache::PCache;
use crate::core::context::ParserContext;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::span::Span;
use crate::error::error_printer::ErrorLabel;
use crate::error::error_printer::ErrorLabel::Debug;
use crate::error::ParseError;

#[inline(always)]
pub fn single<'b, 'grm: 'b, E: ParseError>(
    f: impl Fn(&char) -> bool,
) -> impl Parser<'b, 'grm, (Span, char), E> {
    move |pos: Pos,
          cache: &mut PCache<'b, 'grm, E>,
          _: &ParserContext|
          -> PResult<(Span, char), E> {
        match pos.next(cache.input) {
            // We can parse the character
            (pos_new, Some((span, e))) if f(&e) => PResult::new_ok((span, e), pos, pos_new),
            // Error
            (pos_new, _) => PResult::new_err(E::new(pos.span_to(pos_new)), pos),
        }
    }
}

#[inline(always)]
pub fn seq2<'b, 'grm: 'b, 'a, O1, O2, E: ParseError>(
    p1: &'a impl Parser<'b, 'grm, O1, E>,
    p2: &'a impl Parser<'b, 'grm, O2, E>,
) -> impl Parser<'b, 'grm, (O1, O2), E> + 'a {
    move |stream: Pos,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext|
          -> PResult<(O1, O2), E> {
        let res1 = p1.parse(stream, cache, context);
        let end_pos = res1.end_pos();
        res1.merge_seq(p2.parse(end_pos, cache, context))
    }
}

#[inline(always)]
pub fn choice2<'b, 'grm: 'b, 'a, O, E: ParseError>(
    p1: &'a impl Parser<'b, 'grm, O, E>,
    p2: &'a impl Parser<'b, 'grm, O, E>,
) -> impl Parser<'b, 'grm, O, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E>, context: &ParserContext| -> PResult<O, E> {
        p1.parse(stream, cache, context)
            .merge_choice(p2.parse(stream, cache, context))
    }
}

#[inline(always)]
pub fn repeat_delim<'b, 'grm: 'b, OP, OD, E: ParseError<L = ErrorLabel<'grm>>>(
    item: impl Parser<'b, 'grm, OP, E>,
    delimiter: impl Parser<'b, 'grm, OD, E>,
    min: usize,
    max: Option<usize>,
) -> impl Parser<'b, 'grm, Vec<OP>, E> {
    move |stream: Pos,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext|
          -> PResult<Vec<OP>, E> {
        let mut last_res: PResult<Vec<OP>, E> = PResult::new_empty(vec![], stream);

        for i in 0..max.unwrap_or(usize::MAX) {
            let pos = last_res.end_pos();
            let part = if i == 0 {
                item.parse(pos, cache, context)
            } else {
                seq2(&delimiter, &item)
                    .parse(pos, cache, context)
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
pub fn end<'b, 'grm: 'b, E: ParseError>() -> impl Parser<'b, 'grm, (), E> {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E>, _: &ParserContext| -> PResult<(), E> {
        match stream.next(cache.input) {
            (s, Some(_)) => PResult::new_err(E::new(stream.span_to(s)), stream),
            (s, None) => PResult::new_empty((), s),
        }
    }
}

#[inline(always)]
pub fn positive_lookahead<'b, 'grm: 'b, O, E: ParseError>(
    p: &impl Parser<'b, 'grm, O, E>,
) -> impl Parser<'b, 'grm, O, E> + '_ {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E>, context: &ParserContext| -> PResult<O, E> {
        match p.parse(stream, cache, context) {
            POk(o, _, _, _, err) => POk(o, stream, stream, false, err),
            PErr(e, s) => PErr(e, s),
        }
    }
}

#[inline(always)]
pub fn negative_lookahead<'b, 'grm: 'b, O, E: ParseError>(
    p: &impl Parser<'b, 'grm, O, E>,
) -> impl Parser<'b, 'grm, (), E> + '_ {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E>, context: &ParserContext| -> PResult<(), E> {
        match p.parse(stream, cache, context) {
            POk(_, _, _, _, _) => PResult::new_err(E::new(stream.span_to(stream)), stream),
            PErr(_, _) => PResult::new_ok((), stream, stream),
        }
    }
}