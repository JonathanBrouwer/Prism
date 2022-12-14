use crate::parser_core::error::ParseError;
use crate::parser_core::parser::Parser;
use crate::parser_core::presult::PResult;
use crate::parser_core::presult::PResult::{PErr, POk};
use crate::parser_core::span::Span;
use crate::parser_core::stream::StringStream;

pub fn single<'grm, E: ParseError, Q, F: Fn(&char) -> bool>(
    f: F,
) -> impl Parser<'grm, (Span, char), E, Q> {
    move |pos: StringStream<'grm>, _: &mut Q| -> PResult<'grm, (Span, char), E> {
        match pos.next() {
            (pos_new, Some((span, e))) if f(&e) => PResult::new_ok((span, e), pos_new),
            (pos_new, _) => PResult::new_err(E::new(pos.span_to(pos_new)), pos),
        }
    }
}

pub fn seq2<'grm, 'a, O1, O2, E: ParseError, Q>(
    p1: &'a impl Parser<'grm, O1, E, Q>,
    p2: &'a impl Parser<'grm, O2, E, Q>,
) -> impl Parser<'grm, (O1, O2), E, Q> + 'a {
    move |stream: StringStream<'grm>, cache: &mut Q| -> PResult<'grm, (O1, O2), E> {
        let res1 = p1.parse(stream, cache);
        let stream = res1.get_stream();
        res1.merge_seq(p2.parse(stream, cache))
    }
}

pub fn choice2<'grm, 'a, O, E: ParseError, Q>(
    p1: &'a impl Parser<'grm, O, E, Q>,
    p2: &'a impl Parser<'grm, O, E, Q>,
) -> impl Parser<'grm, O, E, Q> + 'a {
    move |stream: StringStream<'grm>, cache: &mut Q| -> PResult<'grm, O, E> {
        p1.parse(stream, cache)
            .merge_choice(p2.parse(stream, cache))
    }
}

pub fn repeat_delim<
    'grm,
    OP,
    OD,
    E: ParseError,
    Q,
    P: Parser<'grm, OP, E, Q>,
    D: Parser<'grm, OD, E, Q>,
>(
    item: P,
    delimiter: D,
    min: usize,
    max: Option<usize>,
) -> impl Parser<'grm, Vec<OP>, E, Q> {
    move |stream: StringStream<'grm>, cache: &mut Q| -> PResult<'grm, Vec<OP>, E> {
        let mut last_res: PResult<'grm, Vec<OP>, E> = PResult::new_ok(vec![], stream);

        for i in 0..max.unwrap_or(usize::MAX) {
            let pos = last_res.get_stream();
            let part = if i == 0 {
                item.parse(pos, cache)
            } else {
                seq2(&delimiter, &item).parse(pos, cache).map(|x| x.1)
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
            }

            if !should_continue {
                break;
            };
        }

        last_res
    }
}

pub fn end<'grm, E: ParseError, Q>() -> impl Parser<'grm, (), E, Q> {
    move |stream: StringStream<'grm>, _: &mut Q| -> PResult<'grm, (), E> {
        match stream.next() {
            (s, Some(_)) => PResult::new_err(E::new(stream.span_to(s)), stream),
            (s, None) => PResult::new_ok((), s),
        }
    }
}

pub fn positive_lookahead<'grm, O, E: ParseError, Q>(
    p: &impl Parser<'grm, O, E, Q>,
) -> impl Parser<'grm, O, E, Q> + '_ {
    move |stream: StringStream<'grm>, cache: &mut Q| -> PResult<'grm, O, E> {
        match p.parse(stream, cache) {
            POk(o, _, err) => POk(o, stream, err),
            PErr(e, s) => PErr(e, s),
        }
    }
}

pub fn negative_lookahead<'grm, O, E: ParseError, Q>(
    p: &impl Parser<'grm, O, E, Q>,
) -> impl Parser<'grm, (), E, Q> + '_ {
    move |stream: StringStream<'grm>, cache: &mut Q| -> PResult<'grm, (), E> {
        match p.parse(stream, cache) {
            POk(_, _, _) => PResult::new_err(E::new(stream.span_to(stream)), stream),
            PErr(_, _) => PResult::new_ok((), stream),
        }
    }
}
