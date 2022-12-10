use crate::parser::core::error::{err_combine, err_combine_opt, ParseError};
use crate::parser::core::parser::Parser;
use crate::parser::core::presult::PResult::{PErr, POk};
use crate::parser::core::stream::Stream;

#[derive(Clone)]
pub enum PResult<O, E: ParseError, S: Stream> {
    POk(O, S, Option<(E, S)>),
    PErr(E, S),
}

impl<O, E: ParseError, S: Stream> PResult<O, E, S> {
    pub fn new_ok(o: O, s: S) -> Self {
        POk(o, s, None)
    }

    pub fn new_err(e: E, s: S) -> Self {
        PErr(e, s)
    }

    pub fn map<P>(self, f: impl FnOnce(O) -> P) -> PResult<P, E, S> {
        match self {
            POk(o, s, e) => POk(f(o), s, e),
            PErr(err, s) => PErr(err, s),
        }
    }

    pub fn add_label(&mut self, l: E::L) {
        match self {
            POk(_, _, e) => {
                if let Some((e, _)) = e.as_mut() {
                    e.add_label(l);
                }
            }
            PErr(e, _) => {
                e.add_label(l);
            }
        }
    }

    pub fn collapse(self) -> Result<O, E> {
        match self {
            POk(o, _, _) => Ok(o),
            PErr(e, _) => Err(e),
        }
    }

    pub fn is_ok(&self) -> bool {
        match self {
            POk(_, _, _) => true,
            PErr(_, _) => false,
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            POk(_, _, _) => false,
            PErr(_, _) => true,
        }
    }

    pub fn get_stream(&self) -> S {
        match self {
            POk(_, s, _) => *s,
            PErr(_, s) => *s,
        }
    }

    pub fn merge_choice(self, other: Self) -> Self {
        match (self, other) {
            // Left ok
            (ok @ POk(_, _, _), _) => ok,

            // Right ok
            (PErr(ne, ns), POk(s, o, be)) => POk(s, o, err_combine_opt(Some((ne, ns)), be)),

            // If either parsed more input, prioritise that
            (PErr(e1, s1), PErr(e2, s2)) => {
                let (e, s) = err_combine((e1, s1), (e2, s2));
                PErr(e, s)
            }
        }
    }

    pub fn merge_seq<O2>(self, other: PResult<O2, E, S>) -> PResult<(O, O2), E, S> {
        match (self, other) {
            (POk(o1, _, e1), POk(o2, s2, e2)) => POk((o1, o2), s2, err_combine_opt(e1, e2)),
            (POk(_, _, e1), PErr(e2, s2)) => {
                let (e, s) = err_combine_opt(e1, Some((e2, s2))).unwrap();
                PErr(e, s)
            }
            (err @ PErr(_, _), _) => err.map(|_| unreachable!()),
        }
    }

    pub fn merge_seq_opt<O2>(self, other: PResult<O2, E, S>) -> PResult<(O, Option<O2>), E, S> {
        match (self, other) {
            (POk(o1, _, e1), POk(o2, s2, e2)) => POk((o1, Some(o2)), s2, err_combine_opt(e1, e2)),
            (POk(o1, s1, e1), PErr(e2, s2)) => {
                POk((o1, None), s1, err_combine_opt(e1, Some((e2, s2))))
            }
            (err @ PErr(_, _), _) => err.map(|_| unreachable!()),
        }
    }

    pub fn merge_choice_parser<Q, P: Parser<O, S, E, Q>>(
        self,
        other: &P,
        stream: S,
        state: &mut Q,
    ) -> Self {
        //Quick out
        if self.is_ok() {
            return self;
        }

        self.merge_choice(other.parse(stream, state))
    }

    pub fn merge_seq_parser<O2, Q, P2: Parser<O2, S, E, Q>>(
        self,
        other: &P2,
        state: &mut Q,
    ) -> PResult<(O, O2), E, S> {
        //Quick out
        if self.is_err() {
            return self.map(|_| unreachable!());
        }

        let pos = self.get_stream();
        self.merge_seq(other.parse(pos, state))
    }

    pub fn merge_seq_opt_parser<O2, Q, P2: Parser<O2, S, E, Q>>(
        self,
        other: &P2,
        state: &mut Q,
    ) -> (PResult<(O, Option<O2>), E, S>, bool) {
        //Quick out
        if self.is_err() {
            return (self.map(|_| unreachable!()), false);
        }

        let pos = self.get_stream();
        let other_res = other.parse(pos, state);
        let should_continue = other_res.is_ok();
        (self.merge_seq_opt(other_res), should_continue)
    }
}
