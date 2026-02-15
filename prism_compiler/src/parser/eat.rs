use crate::parser::ParserPrismEnv;
use crate::parser::expect::{Expected, PResult};
use prism_input::span::Span;

impl<'a> ParserPrismEnv<'a> {
    /// Takes a single character from the input
    pub fn eat_char(&mut self, f: impl Fn(char) -> bool) -> Result<char, ()> {
        match self.pos.next(&self.db.input) {
            Some((ch, next_pos)) if f(ch) => {
                self.pos = next_pos;
                Ok(ch)
            }
            _ => Err(()),
        }
    }

    pub fn peek_lit_raw(&mut self, lit: &str) -> Result<Span, Span> {
        let start = self.pos;

        // Check if all characters match the input
        for expected_char in lit.chars() {
            let Ok(_) = self.eat_char(|c| c == expected_char) else {
                let fail_pos = self.pos;
                self.pos = start;
                return Err(start.span_to(fail_pos));
            };
        }

        // If lit is a valid ident, the next char cannot be a valid ident char
        // This is to prevent cases like `letx = 5` being valid
        if Self::is_ident(lit) && self.eat_char(unicode_ident::is_xid_continue).is_ok() {
            while self.eat_char(unicode_ident::is_xid_continue).is_ok() {}
            let fail_pos = self.pos;
            self.pos = start;
            return Err(start.span_to(fail_pos));
        }

        Ok(start.span_to(self.pos))
    }

    pub fn eat_lit_raw(&mut self, lit: &str) -> PResult<Span> {
        match self.peek_lit_raw(lit) {
            Ok(res) => Ok(res),
            Err(err_span) => Err(self.expect(err_span, Expected::Literal(lit.to_string()))),
        }
    }

    pub fn eat_lit(&mut self, lit: &str) -> PResult<Span> {
        self.eat_layout();
        self.eat_lit_raw(lit)
    }

    fn is_ident(lit: &str) -> bool {
        let mut chars = lit.chars();
        let Some(first_char) = chars.next() else {
            return false;
        };
        if !unicode_ident::is_xid_start(first_char) {
            return false;
        }
        chars.all(unicode_ident::is_xid_continue)
    }
}
