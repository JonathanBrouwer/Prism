use crate::parser::ParserPrismEnv;
use prism_input::tokens::{Token, TokenType};

impl<'a> ParserPrismEnv<'a> {
    fn eat_comment(&mut self) {
        let start = self.pos;
        while let Ok(_) = self.peek_lit_raw("//") {
            loop {
                let Ok(c) = self.eat_char() else { break };
                if c == '\n' {
                    break;
                }
            }
            self.tokens.push(Token {
                token_type: TokenType::Comment,
                span: start.span_to(self.pos),
            })
        }
    }

    pub fn eat_layout(&mut self) {
        loop {
            self.eat_comment();
            let Some((ch, next_pos)) = self.pos.next(&self.db.input) else {
                return;
            };
            if !ch.is_whitespace() {
                break;
            }
            self.pos = next_pos;
        }
    }
}
