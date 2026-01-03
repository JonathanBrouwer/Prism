use crate::core::context::PV;
use crate::core::presult::PResult;
use crate::core::state::ParserState;
use crate::error::ParseError;
use crate::error::error_label::ErrorLabel;
use crate::parsable::parsed::ArcExt;
use crate::parsable::void::Void;
use prism_input::pos::Pos;
use prism_input::span::Span;
use std::sync::Arc;

impl<Db, E: ParseError<L = ErrorLabel>> ParserState<Db, E> {
    pub fn parse_char(&mut self, f: impl Fn(&char) -> bool, pos: Pos) -> PResult<(Span, char), E> {
        match pos.next(&self.input) {
            // We can parse the character
            (pos_new, Some((span, e))) if f(&e) => PResult::new_ok((span, e), pos, pos_new),
            // Error
            (_, _) => PResult::new_err(E::new(pos), pos),
        }
    }

    /// Parses a literal, error is at `start_pos` if it fails
    pub fn parse_lit(&mut self, lit: &str, start_pos: Pos) -> PResult<(), E> {
        let mut pos = start_pos;
        for char in lit.chars() {
            match pos.next(&self.input) {
                // Literal still matches
                (pos_new, Some((_, c))) if c == char => {
                    pos = pos_new;
                    continue;
                }
                // Literal does not match
                _ => return PResult::new_err(E::new(start_pos), start_pos),
            }
        }
        PResult::new_ok((), start_pos, pos)
    }

    pub fn parse_end(&mut self, pos: Pos) -> PResult<PV, E> {
        match pos.next(&self.input) {
            (_, Some(_)) => PResult::new_err(E::new(pos), pos),
            (s, None) => PResult::new_empty(PV::new_multi(Arc::new(Void).to_parsed(), vec![]), s),
        }
    }
}
