use crate::parser::core::error::ParseError;
use crate::parser::core::parser::Parser;
use crate::parser::core::presult::PResult;
use crate::parser::core::presult::PResult::{PErr, POk, PRec};
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
        p1.parse(stream, state).merge_seq(p2, state)
    }
}

pub fn choice2<'a, I: Clone + Eq, O, S: Stream<I = I>, E: ParseError, Q>(
    p1: &'a impl Parser<I, O, S, E, Q>,
    p2: &'a impl Parser<I, O, S, E, Q>,
) -> impl Parser<I, O, S, E, Q> + 'a {
    move |stream: S, state: &mut Q| -> PResult<O, E, S> {
        p1.parse(stream, state).merge_choice(p2, stream, state)
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

        fn push<T>((mut vec, item): (Vec<T>, T)) -> Vec<T> {
            vec.push(item);
            vec
        }
        fn push_opt<T>((mut vec, item): (Vec<T>, Option<T>)) -> Vec<T> {
            if let Some(item) = item {
                vec.push(item);
            }
            vec
        }

        for i in 0..min {
            if i == 0 {
                last_res = last_res.merge_seq(&item, state).map(push);
            } else {
                last_res = last_res
                    .merge_seq(&seq2(&delimiter, &item), state)
                    .map(|(x, (_, z))| (x, z))
                    .map(push);
            }
            if last_res.is_err() {
                return last_res;
            }
        }

        for i in min..max.unwrap_or(usize::MAX) {
            if i == 0 {
                let (res, should_continue) = last_res.merge_seq_opt(&item, state);
                last_res = res.map(push_opt);
                if !should_continue { break }
            } else {
                let (res, should_continue) = last_res.merge_seq_opt(&seq2(&delimiter, &item), state);
                last_res = res.map(|(x, o)| (x, o.map(|(_, o)| o)))
                    .map(push_opt);
                if !should_continue { break }
            }
        }

        last_res
    }
}

pub fn full_input<'a, I: Clone + Eq, O, S: Stream<I = I>, E: ParseError, Q>(
    p1: &'a impl Parser<I, O, S, E, Q>,
) -> impl Parser<I, O, S, E, Q> + 'a {
    move |stream: S, state: &mut Q| {
        match p1.parse(stream, state) {
            POk(o, rest) => {
                match rest.next().1 {
                    None => POk(o, rest),
                    Some(_) => PErr(vec![], E::new(rest.span_rest()), rest), //TODO best error
                }
            }
            PRec(errs, o, rest) => {
                match rest.next().1 {
                    None => PRec(errs, o, rest),
                    Some(_) => PErr(vec![], E::new(rest.span_rest()), rest), //TODO best error
                }
            }
            err@PErr(_, _, _) => err,
        }
    }
}