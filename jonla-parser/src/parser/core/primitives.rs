use crate::parser::core::error::ParseError;
use crate::parser::core::parser::Parser;
use crate::parser::core::presult::PResult;
use crate::parser::core::presult::PResult::{PErr, POk};
use crate::parser::core::span::Span;
use crate::parser::core::stream::Stream;

pub fn single<S: Stream, E: ParseError, Q, F: Fn(&char) -> bool>(
    f: F,
) -> impl Parser<(Span, char), S, E, Q> {
    move |pos: S, _: &mut Q| -> PResult<(Span, char), E, S> {
        match pos.next() {
            (pos_new, Some((span, e))) if f(&e) => PResult::new_ok((span, e), pos_new),
            (pos_new, _) => PResult::new_err(E::new(pos.span_to(pos_new)), pos),
        }
    }
}

pub fn seq2<'a, O1, O2, S: Stream, E: ParseError, Q>(
    p1: &'a impl Parser<O1, S, E, Q>,
    p2: &'a impl Parser<O2, S, E, Q>,
) -> impl Parser<(O1, O2), S, E, Q> + 'a {
    move |stream: S, state: &mut Q| -> PResult<(O1, O2), E, S> {
        let res1 = p1.parse(stream, state);
        let stream = res1.get_stream();
        res1.merge_seq(p2.parse(stream, state))
    }
}

pub fn choice2<'a, O, S: Stream, E: ParseError, Q>(
    p1: &'a impl Parser<O, S, E, Q>,
    p2: &'a impl Parser<O, S, E, Q>,
) -> impl Parser<O, S, E, Q> + 'a {
    move |stream: S, state: &mut Q| -> PResult<O, E, S> {
        p1.parse(stream, state)
            .merge_choice(p2.parse(stream, state))
    }
}

pub fn repeat_delim<
    OP,
    OD,
    S: Stream,
    E: ParseError,
    Q,
    P: Parser<OP, S, E, Q>,
    D: Parser<OD, S, E, Q>,
>(
    item: P,
    delimiter: D,
    min: usize,
    max: Option<usize>,
) -> impl Parser<Vec<OP>, S, E, Q> {
    move |stream: S, state: &mut Q| -> PResult<Vec<OP>, E, S> {
        let mut last_res: PResult<Vec<OP>, E, S> = PResult::new_ok(vec![], stream);

        for i in 0..max.unwrap_or(usize::MAX) {
            let pos = last_res.get_stream();
            let part = if i == 0 {
                item.parse(pos, state)
            } else {
                seq2(&delimiter, &item).parse(pos, state).map(|x| x.1)
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

pub fn end<'a, S: Stream, E: ParseError, Q>() -> impl Parser<(), S, E, Q> + 'a {
    move |stream: S, _: &mut Q| -> PResult<(), E, S> {
        match stream.next() {
            (s, Some(_)) => PResult::new_err(E::new(stream.span_to(s)), stream),
            (s, None) => PResult::new_ok((), s),
        }
    }
}

pub fn positive_lookahead<'a, O, S: Stream, E: ParseError, Q>(
    p: &'a impl Parser<O, S, E, Q>,
) -> impl Parser<O, S, E, Q> + 'a {
    move |stream: S, state: &mut Q| -> PResult<O, E, S> {
        match p.parse(stream, state) {
            POk(o, _, err) => POk(o, stream, err),
            PErr(e, s) => PErr(e, s),
        }
    }
}

pub fn negative_lookahead<'a, O, S: Stream, E: ParseError, Q>(
    p: &'a impl Parser<O, S, E, Q>,
) -> impl Parser<(), S, E, Q> + 'a {
    move |stream: S, state: &mut Q| -> PResult<(), E, S> {
        match p.parse(stream, state) {
            POk(_, _, _) => PResult::new_err(E::new(stream.span_to(stream)), stream),
            PErr(_, _) => PResult::new_ok((), stream),
        }
    }
}
