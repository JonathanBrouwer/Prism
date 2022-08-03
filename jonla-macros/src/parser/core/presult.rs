use crate::parser::core::error::ParseError;
use crate::parser::core::parser::Parser;
use crate::parser::core::presult::PResult::{PErr, POk, PRec};
use crate::parser::core::stream::Stream;
use std::cmp::Ordering;

#[derive(Clone)]
pub enum PResult<O, E: ParseError, S: Stream> {
    POk(O, S),
    PRec(Vec<E>, O, S),
    PErr(Vec<E>, E, S),
}

impl<O, E: ParseError, S: Stream> PResult<O, E, S> {
    pub fn new_ok(o: O, s: S) -> Self {
        POk(o, s)
    }

    pub fn new_err(e: E, s: S) -> Self {
        PErr(vec![], e, s)
    }

    pub fn map<P>(self, f: impl FnOnce(O) -> P) -> PResult<P, E, S> {
        match self {
            POk(o, s) => POk(f(o), s),
            PRec(errs, o, s) => PRec(errs, f(o), s),
            PErr(errs, err, s) => PErr(errs, err, s),
        }
    }

    pub fn collapse(self) -> (Vec<E>, Option<O>) {
        match self {
            POk(o, _) => (vec![], Some(o)),
            PRec(es, o, _) => (es, Some(o)),
            PErr(mut es, e, _) => {
                es.push(e);
                (es, None)
            }
        }
    }

    pub fn is_ok(&self) -> bool {
        match self {
            POk(_, _) => true,
            PRec(_, _, _) => false,
            PErr(_, _, _) => false,
        }
    }

    pub fn is_rec(&self) -> bool {
        match self {
            POk(_, _) => false,
            PRec(_, _, _) => true,
            PErr(_, _, _) => false,
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            POk(_, _) => false,
            PRec(_, _, _) => false,
            PErr(_, _, _) => true,
        }
    }

    pub fn get_stream(&self) -> S {
        match self {
            POk(_, s) => *s,
            PRec(_, _, s) => *s,
            PErr(_, _, s) => *s,
        }
    }

    pub fn merge_choice<Q, P: Parser<S::I, O, S, E, Q>>(
        self,
        other: &P,
        stream: S,
        state: &mut Q,
    ) -> Self {
        //Quick out
        if self.is_ok() {
            return self;
        }

        let other = other.parse(stream, state);
        let cmp = self.get_stream().cmp(other.get_stream());
        match (self, cmp, other) {
            // If either is ok, prioritise that
            (POk(_, _), _, _) => unreachable!(),
            (_, _, POk(s, o)) => POk(s, o),

            // If either parsed more input, prioritise that
            (r, Ordering::Greater, _) => r,
            (_, Ordering::Less, r) => r,

            // Combine
            (PErr(errs1, err1, s), Ordering::Equal, PErr(errs2, err2, _))
                if errs1.is_empty() && errs2.is_empty() =>
            {
                PErr(vec![], err1.merge(err2), s)
            }

            // Ok I give up, just choose the first one
            (pr1, Ordering::Equal, _) => pr1,
        }
    }

    pub fn merge_seq<O2, Q, P2: Parser<S::I, O2, S, E, Q>>(
        self,
        other: &P2,
        state: &mut Q,
    ) -> PResult<(O, O2), E, S> {
        fn conc<E>(mut vec1: Vec<E>, mut vec2: Vec<E>) -> Vec<E> {
            vec1.append(&mut vec2);
            vec1
        }

        //Quick out
        if self.is_err() {
            return self.map(|_| unreachable!());
        }

        let stream = self.get_stream();
        match (self, other.parse(stream, state)) {
            (POk(o1, _), POk(o2, s2)) => POk((o1, o2), s2),
            (POk(o1, _), PRec(errs2, o2, s2)) => PRec(errs2, (o1, o2), s2),
            (POk(_, _), PErr(errs2, err2, s2)) => PErr(errs2, err2, s2),
            (PRec(errs1, o1, _), POk(o2, s2)) => PRec(errs1, (o1, o2), s2),
            (PRec(errs1, o1, _), PRec(errs2, o2, s2)) => PRec(conc(errs1, errs2), (o1, o2), s2),
            (PRec(errs1, _, _), PErr(errs2, err2, s2)) => PErr(conc(errs1, errs2), err2, s2),
            (PErr(_, _, _), _) => unreachable!(),
        }
    }

    pub fn merge_seq_opt<O2, Q, P2: Parser<S::I, O2, S, E, Q>>(
        self,
        other: &P2,
        state: &mut Q,
    ) -> (PResult<(O, Option<O2>), E, S>, bool) {
        fn conc<E>(mut vec1: Vec<E>, mut vec2: Vec<E>) -> Vec<E> {
            vec1.append(&mut vec2);
            vec1
        }

        //Quick out
        if self.is_err() {
            return (self.map(|_| unreachable!()), false);
        }

        let stream = self.get_stream();
        match (self, other.parse(stream, state)) {
            (POk(o1, _), POk(o2, s2)) => (POk((o1, Some(o2)), s2), true),
            (POk(o1, s1), _) => (POk((o1, None), s1), false),
            (PRec(errs1, o1, _), POk(o2, s2)) => (PRec(errs1, (o1, Some(o2)), s2), true),
            (PRec(errs1, o1, _), PRec(errs2, o2, s2)) => {
                (PRec(conc(errs1, errs2), (o1, Some(o2)), s2), true)
            }
            (PRec(errs1, _, _), PErr(errs2, err2, s2)) => (PErr(conc(errs1, errs2), err2, s2), false),
            (PErr(_, _, _), _) => unreachable!(),
        }
    }
}
