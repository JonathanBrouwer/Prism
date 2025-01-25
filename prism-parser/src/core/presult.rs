use crate::core::pos::Pos;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::span::Span;
use crate::error::{ParseError, err_combine, err_combine_opt};

#[derive(Clone)]
pub enum PResult<O, E: ParseError> {
    POk(O, Pos, Pos, Option<(E, Pos)>),
    PErr(E, Pos),
}

impl<O, E: ParseError> PResult<O, E> {
    pub fn new_empty(o: O, pos: Pos) -> Self {
        POk(o, pos, pos, None)
    }

    pub fn new_ok(o: O, start: Pos, end: Pos) -> Self {
        POk(o, start, end, None)
    }

    pub fn new_err(e: E, s: Pos) -> Self {
        PErr(e, s)
    }

    pub fn map<P>(self, f: impl FnOnce(O) -> P) -> PResult<P, E> {
        match self {
            POk(o, start, end, e) => POk(f(o), start, end, e),
            PErr(err, s) => PErr(err, s),
        }
    }

    pub fn map_with_span<P>(self, f: impl FnOnce(O, Span) -> P) -> PResult<P, E> {
        match self {
            POk(o, start, end, e) => POk(f(o, start.span_to(end)), start, end, e),
            PErr(err, s) => PErr(err, s),
        }
    }

    pub fn add_label_explicit(&mut self, l: E::L) {
        match self {
            POk(_, _, _, e) => {
                if let Some((e, _)) = e.as_mut() {
                    e.add_label_explicit(l);
                }
            }
            PErr(e, _) => {
                e.add_label_explicit(l);
            }
        }
    }

    pub fn add_label_implicit(&mut self, l: E::L) {
        match self {
            POk(_, _, _, e) => {
                if let Some((e, _)) = e.as_mut() {
                    e.add_label_implicit(l);
                }
            }
            PErr(e, _) => {
                e.add_label_implicit(l);
            }
        }
    }

    pub fn collapse(self) -> Result<O, E> {
        match self {
            POk(o, _, _, _) => Ok(o),
            PErr(e, _) => Err(e),
        }
    }

    pub fn is_ok(&self) -> bool {
        match self {
            POk(_, _, _, _) => true,
            PErr(_, _) => false,
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            POk(_, _, _, _) => false,
            PErr(_, _) => true,
        }
    }

    pub fn end_pos(&self) -> Pos {
        match self {
            POk(_, _, s, _) => *s,
            PErr(_, s) => *s,
        }
    }

    pub fn merge_choice(self, other: Self) -> Self {
        match (self, other) {
            // Left ok
            (ok @ POk(_, _, _, _), _) => ok,

            // Right ok
            (PErr(ne, ns), POk(s, start, end, be)) => {
                POk(s, start, end, err_combine_opt(Some((ne, ns)), be))
            }

            // If either parsed more input, prioritise that
            (PErr(e1, s1), PErr(e2, s2)) => {
                let (e, s) = err_combine((e1, s1), (e2, s2));
                PErr(e, s)
            }
        }
    }

    pub fn merge_seq<O2>(self, other: PResult<O2, E>) -> PResult<(O, O2), E> {
        match (self, other) {
            (POk(o1, start1, end1, e1), POk(o2, start2, end2, e2)) => {
                // If the first result is empty and the second is not, we skip the first
                let start = if start1 == end1 { start2 } else { start1 };
                POk((o1, o2), start, end2, err_combine_opt(e1, e2))
            }
            (POk(_, _, _, e1), PErr(e2, s2)) => {
                let (e, s) = err_combine_opt(e1, Some((e2, s2))).unwrap();
                PErr(e, s)
            }
            (err @ PErr(_, _), _) => err.map(|_| unreachable!()),
        }
    }

    pub fn merge_seq_opt<O2>(self, other: PResult<O2, E>) -> PResult<(O, Option<O2>), E> {
        match (self, other) {
            (r1 @ POk(_, _, _, _), r2 @ POk(_, _, _, _)) => {
                r1.merge_seq(r2).map(|(o1, o2)| (o1, Some(o2)))
            }
            (POk(o1, start, end, e1), PErr(e2, s2)) => {
                POk((o1, None), start, end, err_combine_opt(e1, Some((e2, s2))))
            }
            (err @ PErr(_, _), _) => err.map(|_| unreachable!()),
        }
    }

    pub fn merge_choice_chain<'arn, 'grm>(self, mut other: impl FnMut() -> PResult<O, E>) -> Self
    where
        'grm: 'arn,
    {
        //Quick out
        if self.is_ok() {
            return self;
        }

        self.merge_choice(other())
    }

    pub fn merge_seq_chain<'arn, 'grm, O2>(
        self,
        mut other: impl FnMut(Pos) -> PResult<O2, E>,
    ) -> PResult<(O, O2), E>
    where
        'grm: 'arn,
    {
        //Quick out
        if self.is_err() {
            return self.map(|_| unreachable!());
        }

        let pos = self.end_pos();
        self.merge_seq(other(pos))
    }

    pub fn merge_seq_chain2<'arn, 'grm, O2>(
        self,
        mut other: impl FnMut(Pos, Span, O) -> PResult<O2, E>,
    ) -> PResult<O2, E>
    where
        'grm: 'arn,
    {
        //Quick out
        match self {
            POk(o, start, end, best) => POk((), start, end, best)
                .merge_seq(other(end, start.span_to(end), o))
                .map(|((), o)| o),
            PErr(_, _) => self.map(|_| unreachable!()),
        }
    }

    pub fn ok(self) -> Option<O> {
        match self {
            POk(o, _, _, _) => Some(o),
            PErr(_, _) => None,
        }
    }

    pub fn ok_ref(&self) -> Option<&O> {
        match self {
            POk(o, _, _, _) => Some(o),
            PErr(_, _) => None,
        }
    }

    pub fn positive_lookahead(self, start_pos: Pos) -> Self {
        match self {
            POk(o, _, _, err) => POk(o, start_pos, start_pos, err),
            PErr(e, s) => PErr(e, s),
        }
    }

    pub fn negative_lookahead(self, start_pos: Pos) -> PResult<(), E> {
        match self {
            POk(_, _, _, _) => PResult::new_err(E::new(start_pos), start_pos),
            PErr(_, _) => PResult::new_empty((), start_pos),
        }
    }
}
