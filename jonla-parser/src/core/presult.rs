use crate::core::context::{PCache, ParserContext};
use crate::core::parser::Parser;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::pos::Pos;
use crate::error::{err_combine, err_combine_opt, ParseError};

#[derive(Clone)]
pub enum PResult<O, E: ParseError> {
    POk(O, Pos, Option<(E, Pos)>),
    PErr(E, Pos),
}

impl<O, E: ParseError> PResult<O, E> {
    #[inline(always)]
    pub fn new_ok(o: O, s: Pos) -> Self {
        POk(o, s, None)
    }

    #[inline(always)]
    pub fn new_err(e: E, s: Pos) -> Self {
        PErr(e, s)
    }

    #[inline(always)]
    pub fn map<P>(self, f: impl FnOnce(O) -> P) -> PResult<P, E> {
        match self {
            POk(o, s, e) => POk(f(o), s, e),
            PErr(err, s) => PErr(err, s),
        }
    }

    #[inline(always)]
    pub fn add_label_explicit(&mut self, l: E::L) {
        match self {
            POk(_, _, e) => {
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
            POk(_, _, e) => {
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
            POk(o, _, _) => Ok(o),
            PErr(e, _) => Err(e),
        }
    }

    #[inline(always)]
    pub fn is_ok(&self) -> bool {
        match self {
            POk(_, _, _) => true,
            PErr(_, _) => false,
        }
    }

    #[inline(always)]
    pub fn is_err(&self) -> bool {
        match self {
            POk(_, _, _) => false,
            PErr(_, _) => true,
        }
    }

    #[inline(always)]
    pub fn get_pos(&self) -> Pos {
        match self {
            POk(_, s, _) => *s,
            PErr(_, s) => *s,
        }
    }

    #[inline(always)]
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

    #[inline(always)]
    pub fn merge_seq<O2>(self, other: PResult<O2, E>) -> PResult<(O, O2), E> {
        match (self, other) {
            (POk(o1, _, e1), POk(o2, s2, e2)) => POk((o1, o2), s2, err_combine_opt(e1, e2)),
            (POk(_, _, e1), PErr(e2, s2)) => {
                let (e, s) = err_combine_opt(e1, Some((e2, s2))).unwrap();
                PErr(e, s)
            }
            (err @ PErr(_, _), _) => err.map(|_| unreachable!()),
        }
    }

    #[inline(always)]
    pub fn merge_seq_opt<O2>(
        self,
        other: PResult<O2, E>,
    ) -> PResult<(O, Option<O2>), E> {
        match (self, other) {
            (POk(o1, _, e1), POk(o2, s2, e2)) => POk((o1, Some(o2)), s2, err_combine_opt(e1, e2)),
            (POk(o1, s1, e1), PErr(e2, s2)) => {
                POk((o1, None), s1, err_combine_opt(e1, Some((e2, s2))))
            }
            (err @ PErr(_, _), _) => err.map(|_| unreachable!()),
        }
    }

    #[inline(always)]
    pub fn merge_choice_parser<'grm, 'b, P: Parser<'b, 'grm, O, E>>(
        self,
        other: &P,
        stream: Pos,
        cache: &mut PCache<'b, 'grm, E>,
        context: &ParserContext<'b, 'grm>,
    ) -> Self
    where
        'grm: 'b,
    {
        //Quick out
        if self.is_ok() {
            return self;
        }

        self.merge_choice(other.parse(stream, cache, context))
    }

    #[inline(always)]
    pub fn merge_seq_parser<'grm, 'b, O2, P2: Parser<'b, 'grm, O2, E>>(
        self,
        other: &P2,
        cache: &mut PCache<'b, 'grm, E>,
        context: &ParserContext<'b, 'grm>,
    ) -> PResult<(O, O2), E>
    where
        'grm: 'b,
    {
        //Quick out
        if self.is_err() {
            return self.map(|_| unreachable!());
        }

        let pos = self.get_pos();
        self.merge_seq(other.parse(pos, cache, context))
    }

    #[inline(always)]
    pub fn merge_seq_opt_parser<'grm, 'b, O2, P2: Parser<'b, 'grm, O2, E>>(
        self,
        other: &P2,
        cache: &mut PCache<'b, 'grm, E>,
        context: &ParserContext<'b, 'grm>,
    ) -> (PResult<(O, Option<O2>), E>, bool)
    where
        'grm: 'b,
    {
        //Quick out
        if self.is_err() {
            return (self.map(|_| unreachable!()), false);
        }

        let pos = self.get_pos();
        let other_res = other.parse(pos, cache, context);
        let should_continue = other_res.is_ok();
        (self.merge_seq_opt(other_res), should_continue)
    }

    #[inline(always)]
    pub fn ok(&self) -> Option<&O> {
        match self {
            POk(o, _, _) => Some(o),
            PErr(_, _) => None,
        }
    }
}
