use crate::core::pos::Pos;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::span::Span;
use crate::error::{ParseError, err_combine, err_combine_opt};

#[derive(Clone)]
pub enum PResult<O, E: ParseError> {
    POk {
        obj: O,
        start: Pos,
        end: Pos,
        best_err: Option<(E, Pos)>,
    },
    PErr {
        err: E,
        end: Pos,
    },
}

impl<O, E: ParseError> PResult<O, E> {
    pub fn new_empty(o: O, pos: Pos) -> Self {
        POk {
            obj: o,
            start: pos,
            end: pos,
            best_err: None,
        }
    }

    pub fn new_ok(o: O, start: Pos, end: Pos) -> Self {
        POk {
            obj: o,
            start,
            end,
            best_err: None,
        }
    }

    pub fn new_err(e: E, s: Pos) -> Self {
        PErr { err: e, end: s }
    }

    pub fn map<P>(self, f: impl FnOnce(O) -> P) -> PResult<P, E> {
        match self {
            POk {
                obj: o,
                start,
                end,
                best_err: e,
            } => POk {
                obj: f(o),
                start,
                end,
                best_err: e,
            },
            PErr { err, end: s } => PErr { err, end: s },
        }
    }

    pub fn map_with_span<P>(self, f: impl FnOnce(O, Span) -> P) -> PResult<P, E> {
        match self {
            POk {
                obj: o,
                start,
                end,
                best_err: e,
            } => POk {
                obj: f(o, start.span_to(end)),
                start,
                end,
                best_err: e,
            },
            PErr { err, end: s } => PErr { err, end: s },
        }
    }

    pub fn add_label_explicit(&mut self, l: E::L) {
        match self {
            POk {
                obj: _,
                start: _,
                end: _,
                best_err: e,
            } => {
                if let Some((e, _)) = e.as_mut() {
                    e.add_label_explicit(l);
                }
            }
            PErr { err: e, end: _ } => {
                e.add_label_explicit(l);
            }
        }
    }

    pub fn add_label_implicit(&mut self, l: E::L) {
        match self {
            POk {
                obj: _,
                start: _,
                end: _,
                best_err: e,
            } => {
                if let Some((e, _)) = e.as_mut() {
                    e.add_label_implicit(l);
                }
            }
            PErr { err: e, end: _ } => {
                e.add_label_implicit(l);
            }
        }
    }

    pub fn collapse(self) -> Result<O, E> {
        match self {
            POk {
                obj: o,
                start: _,
                end: _,
                best_err: _,
            } => Ok(o),
            PErr { err: e, end: _ } => Err(e),
        }
    }

    pub fn is_ok(&self) -> bool {
        match self {
            POk {
                obj: _,
                start: _,
                end: _,
                best_err: _,
            } => true,
            PErr { err: _, end: _ } => false,
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            POk {
                obj: _,
                start: _,
                end: _,
                best_err: _,
            } => false,
            PErr { err: _, end: _ } => true,
        }
    }

    pub fn end_pos(&self) -> Pos {
        match self {
            POk {
                obj: _,
                start: _,
                end: s,
                best_err: _,
            } => *s,
            PErr { err: _, end: s } => *s,
        }
    }

    pub fn merge_choice(self, other: Self) -> Self {
        match (self, other) {
            // Left ok
            (
                ok @ POk {
                    obj: _,
                    start: _,
                    end: _,
                    best_err: _,
                },
                _,
            ) => ok,

            // Right ok
            (
                PErr { err: ne, end: ns },
                POk {
                    obj: s,
                    start,
                    end,
                    best_err: be,
                },
            ) => POk {
                obj: s,
                start,
                end,
                best_err: err_combine_opt(Some((ne, ns)), be),
            },

            // If either parsed more input, prioritise that
            (PErr { err: e1, end: s1 }, PErr { err: e2, end: s2 }) => {
                let (e, s) = err_combine((e1, s1), (e2, s2));
                PErr { err: e, end: s }
            }
        }
    }

    pub fn merge_seq<O2>(self, other: PResult<O2, E>) -> PResult<(O, O2), E> {
        match (self, other) {
            (
                POk {
                    obj: o1,
                    start: start1,
                    end: end1,
                    best_err: e1,
                },
                POk {
                    obj: o2,
                    start: start2,
                    end: end2,
                    best_err: e2,
                },
            ) => {
                // If the first result is empty and the second is not, we skip the first
                let start = if start1 == end1 { start2 } else { start1 };
                POk {
                    obj: (o1, o2),
                    start,
                    end: end2,
                    best_err: err_combine_opt(e1, e2),
                }
            }
            (
                POk {
                    obj: _,
                    start: _,
                    end: _,
                    best_err: e1,
                },
                PErr { err: e2, end: s2 },
            ) => {
                let (e, s) = err_combine_opt(e1, Some((e2, s2))).unwrap();
                PErr { err: e, end: s }
            }
            (err @ PErr { err: _, end: _ }, _) => err.map(|_| unreachable!()),
        }
    }

    pub fn merge_seq_opt<O2>(self, other: PResult<O2, E>) -> PResult<(O, Option<O2>), E> {
        match (self, other) {
            (
                r1 @ POk {
                    obj: _,
                    start: _,
                    end: _,
                    best_err: _,
                },
                r2 @ POk {
                    obj: _,
                    start: _,
                    end: _,
                    best_err: _,
                },
            ) => r1.merge_seq(r2).map(|(o1, o2)| (o1, Some(o2))),
            (
                POk {
                    obj: o1,
                    start,
                    end,
                    best_err: e1,
                },
                PErr { err: e2, end: s2 },
            ) => POk {
                obj: (o1, None),
                start,
                end,
                best_err: err_combine_opt(e1, Some((e2, s2))),
            },
            (err @ PErr { err: _, end: _ }, _) => err.map(|_| unreachable!()),
        }
    }

    pub fn merge_choice_chain(self, mut other: impl FnMut() -> PResult<O, E>) -> Self {
        //Quick out
        if self.is_ok() {
            return self;
        }

        self.merge_choice(other())
    }

    pub fn merge_seq_chain<O2>(
        self,
        mut other: impl FnMut(Pos) -> PResult<O2, E>,
    ) -> PResult<(O, O2), E> {
        //Quick out
        if self.is_err() {
            return self.map(|_| unreachable!());
        }

        let pos = self.end_pos();
        self.merge_seq(other(pos))
    }

    pub fn merge_seq_chain2<O2>(
        self,
        mut other: impl FnMut(Pos, Span, O) -> PResult<O2, E>,
    ) -> PResult<O2, E> {
        //Quick out
        match self {
            POk {
                obj: o,
                start,
                end,
                best_err: best,
            } => POk {
                obj: (),
                start,
                end,
                best_err: best,
            }
            .merge_seq(other(end, start.span_to(end), o))
            .map(|((), o)| o),
            PErr { err: _, end: _ } => self.map(|_| unreachable!()),
        }
    }

    pub fn ok(self) -> Option<O> {
        match self {
            POk {
                obj: o,
                start: _,
                end: _,
                best_err: _,
            } => Some(o),
            PErr { err: _, end: _ } => None,
        }
    }

    pub fn ok_ref(&self) -> Option<&O> {
        match self {
            POk {
                obj: o,
                start: _,
                end: _,
                best_err: _,
            } => Some(o),
            PErr { err: _, end: _ } => None,
        }
    }

    pub fn positive_lookahead(self, start_pos: Pos) -> Self {
        match self {
            POk {
                obj: o,
                start: _,
                end: _,
                best_err: err,
            } => POk {
                obj: o,
                start: start_pos,
                end: start_pos,
                best_err: err,
            },
            PErr { err: e, end: s } => PErr { err: e, end: s },
        }
    }

    pub fn negative_lookahead(self, start_pos: Pos) -> PResult<(), E> {
        match self {
            POk {
                obj: _,
                start: _,
                end: _,
                best_err: _,
            } => PResult::new_err(E::new(start_pos), start_pos),
            PErr { err: _, end: _ } => PResult::new_empty((), start_pos),
        }
    }
}
