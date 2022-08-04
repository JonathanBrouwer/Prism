use crate::parser::core::error::ParseError;
use crate::parser::core::parser::Parser;
use crate::parser::core::presult::PResult;
use crate::parser::core::presult::PResult::{PErr, POk};
use crate::parser::core::span::Span;
use crate::parser::core::stream::Stream;

pub fn single<I: Clone + Eq, S: Stream<I = I>, E: ParseError, Q, F: Fn(&I) -> bool>(
    f: F,
) -> impl Parser<I, (Span, I), S, E, Q> {
    move |pos: S, _: &mut Q| -> PResult<(Span, I), E, S> {
        match pos.next() {
            (pos_new, Some((span, e))) if f(&e) => PResult::new_ok((span, e), pos_new),
            (pos_new, _) => PResult::new_err(E::new(pos.span_to(pos_new)), pos),
        }
    }
}

pub fn seq2<'a, I: Clone + Eq, O1, O2, S: Stream<I = I>, E: ParseError, Q>(
    p1: &'a impl Parser<I, O1, S, E, Q>,
    p2: &'a impl Parser<I, O2, S, E, Q>,
) -> impl Parser<I, (O1, O2), S, E, Q> + 'a {
    move |stream: S, state: &mut Q| -> PResult<(O1, O2), E, S> {
        let res1 = p1.parse(stream, state);
        let stream = res1.get_stream();
        res1.merge_seq(p2.parse(stream, state))
    }
}

pub fn choice2<'a, I: Clone + Eq, O, S: Stream<I = I>, E: ParseError, Q>(
    p1: &'a impl Parser<I, O, S, E, Q>,
    p2: &'a impl Parser<I, O, S, E, Q>,
) -> impl Parser<I, O, S, E, Q> + 'a {
    move |stream: S, state: &mut Q| -> PResult<O, E, S> {
        p1.parse(stream, state).merge_choice(p2.parse(stream, state))
    }
}

pub fn repeat_delim<
    I: Clone + Eq,
    OP,
    OD,
    S: Stream<I = I>,
    E: ParseError,
    Q,
    P: Parser<I, OP, S, E, Q>,
    D: Parser<I, OD, S, E, Q>,
>(
    item: P,
    delimiter: D,
    min: usize,
    max: Option<usize>,
) -> impl Parser<I, Vec<OP>, S, E, Q> {
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

            if !should_continue {break};
        }

        last_res
    }
}

pub fn full_input<'a, I: Clone + Eq, O, S: Stream<I = I>, E: ParseError, Q>(
    p1: &'a impl Parser<I, O, S, E, Q>,
) -> impl Parser<I, O, S, E, Q> + 'a {
    move |stream: S, state: &mut Q| {
        match p1.parse(stream, state) {
            POk(o, rest, be) => {
                match (rest.next().1, be) {
                    (None, be) => POk(o, rest, be),
                    (Some(_), Some((be, bs))) => PErr(be, bs),
                    (Some(_), None) => PErr(E::new(rest.span_rest()), rest), //TODO explain what happened
                }
            }
            err @ PErr(_, _) => err,
        }
    }
}
