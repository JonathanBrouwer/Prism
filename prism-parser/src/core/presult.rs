use crate::core::context::ParserContext;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::span::Span;
use crate::core::state::PState;
use crate::error::{err_combine, err_combine_opt, ParseError};

#[derive(Clone)]
pub enum PResult<O, E: ParseError> {
    POk(O, Pos, Pos, Option<(E, Pos)>),
    PErr(E, Pos),
}

impl<O, E: ParseError> PResult<O, E> {
    #[inline(always)]
    pub fn new_empty(o: O, pos: Pos) -> Self {
        POk(o, pos, pos, None)
    }

    #[inline(always)]
    pub fn new_ok(o: O, start: Pos, end: Pos) -> Self {
        POk(o, start, end, None)
    }

    #[inline(always)]
    pub fn new_err(e: E, s: Pos) -> Self {
        PErr(e, s)
    }

    #[inline(always)]
    pub fn map<P>(self, f: impl FnOnce(O) -> P) -> PResult<P, E> {
        match self {
            POk(o, start, end, e) => POk(f(o), start, end, e),
            PErr(err, s) => PErr(err, s),
        }
    }

    #[inline(always)]
    pub fn map_with_span<P>(self, f: impl FnOnce(O, Span) -> P) -> PResult<P, E> {
        match self {
            POk(o, start, end, e) => POk(f(o, start.span_to(end)), start, end, e),
            PErr(err, s) => PErr(err, s),
        }
    }

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
    pub fn collapse(self) -> Result<O, E> {
        match self {
            POk(o, _, _, _) => Ok(o),
            PErr(e, _) => Err(e),
        }
    }

    #[inline(always)]
    pub fn is_ok(&self) -> bool {
        match self {
            POk(_, _, _, _) => true,
            PErr(_, _) => false,
        }
    }

    #[inline(always)]
    pub fn is_err(&self) -> bool {
        match self {
            POk(_, _, _, _) => false,
            PErr(_, _) => true,
        }
    }

    #[inline(always)]
    pub fn end_pos(&self) -> Pos {
        match self {
            POk(_, _, s, _) => *s,
            PErr(_, s) => *s,
        }
    }

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
    pub fn merge_choice_parser<'arn, 'grm, P: Parser<'arn, 'grm, O, E>>(
        self,
        other: &P,
        pos: Pos,
        state: &mut PState<'arn, 'grm, E>,
        context: ParserContext,
    ) -> Self
    where
        'grm: 'arn,
    {
        //Quick out
        if self.is_ok() {
            return self;
        }

        self.merge_choice(other.parse(pos, state, context))
    }

    #[inline(always)]
    pub fn merge_seq_parser<'arn, 'grm, O2, P2: Parser<'arn, 'grm, O2, E>>(
        self,
        other: &P2,
        state: &mut PState<'arn, 'grm, E>,
        context: ParserContext,
    ) -> PResult<(O, O2), E>
    where
        'grm: 'arn,
    {
        //TODO write as  self.merge_seq_chain_parser(|o1| map_parser(other, &|o2| (o1, o2)), state, context)
        //Quick out
        if self.is_err() {
            return self.map(|_| unreachable!());
        }

        let pos = self.end_pos();
        self.merge_seq(other.parse(pos, state, context))
    }

    #[inline(always)]
    pub fn merge_seq_opt_parser<'arn, 'grm, O2, P2: Parser<'arn, 'grm, O2, E>>(
        self,
        other: &P2,
        state: &mut PState<'arn, 'grm, E>,
        context: ParserContext,
    ) -> (PResult<(O, Option<O2>), E>, bool)
    where
        'grm: 'arn,
    {
        //Quick out
        if self.is_err() {
            return (self.map(|_| unreachable!()), false);
        }

        let pos = self.end_pos();
        let other_res = other.parse(pos, state, context);
        let should_continue = other_res.is_ok();
        (self.merge_seq_opt(other_res), should_continue)
    }

    #[inline(always)]
    pub fn merge_seq_chain_parser<'arn, 'grm, O2, P2: Parser<'arn, 'grm, O2, E>>(
        self,
        other: impl FnOnce(O) -> P2,
        state: &mut PState<'arn, 'grm, E>,
        context: ParserContext,
    ) -> PResult<O2, E>
    where
        'grm: 'arn,
    {
        match self {
            POk(o, _, end_pos, _) => other(o).parse(end_pos, state, context),
            PErr(_, _) => self.map(|_| unreachable!()),
        }
    }

    #[inline(always)]
    pub fn ok(self) -> Option<O> {
        match self {
            POk(o, _, _, _) => Some(o),
            PErr(_, _) => None,
        }
    }

    #[inline(always)]
    pub fn ok_ref(&self) -> Option<&O> {
        match self {
            POk(o, _, _, _) => Some(o),
            PErr(_, _) => None,
        }
    }
}
