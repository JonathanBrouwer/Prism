use crate::parser::ParserPrismEnv;
use crate::parser::expect::{Expected, PResult};
use crate::parser::lexer::{SYMBOL_CHARS, Token};
use prism_input::span::Span;

impl<'a> ParserPrismEnv<'a> {
    pub fn eat_token(
        &mut self,
        expected: Expected,
        f: impl Fn(Token, &mut Self) -> bool,
    ) -> PResult<Token> {
        let token = self.next_token();
        if f(token, self) {
            Ok(token)
        } else {
            self.pop_token();
            Err(self.expect(token.span(), expected))
        }
    }

    pub fn eat_keyword(&mut self, expected_str: &str) -> PResult<Span> {
        self.eat_token(Expected::Literal(expected_str.to_string()), |token, env| {
            if let Token::Identifier { span, .. } = token
                && env.db.input.inner().slice(span) == expected_str
            {
                env.mark_token_keyword();
                true
            } else {
                false
            }
        })
        .map(|tok| tok.span())
    }

    pub fn eat_paren_open(&mut self, open: &str) -> PResult<Span> {
        self.eat_token(Expected::Literal(open.to_string()), |token, env| {
            if let Token::OpenParen(span) = token
                && env.db.input.inner().slice(span) == open
            {
                true
            } else {
                false
            }
        })
        .map(|t| t.span())
    }

    pub fn eat_paren_close(&mut self, close: &str) -> PResult<Span> {
        self.eat_token(Expected::Literal(close.to_string()), |token, env| {
            if let Token::CloseParen(span) = token
                && env.db.input.inner().slice(span) == close
            {
                true
            } else {
                false
            }
        })
        .map(|t| t.span())
    }

    pub fn eat_identifier(&mut self) -> PResult<Span> {
        self.eat_token(Expected::Rule("identifier".to_string()), |token, _env| {
            matches!(token, Token::Identifier { .. })
        })
        .map(|tok| tok.span())
    }

    pub fn eat_symbol(&mut self, expected_symbol: char) -> PResult<Span> {
        assert!(SYMBOL_CHARS.contains(expected_symbol));
        self.eat_token(
            Expected::Literal(expected_symbol.to_string()),
            |token, env| {
                if let Token::Symbol(span) = token
                    && env.db.input.inner().slice(span) == String::from_iter([expected_symbol])
                {
                    true
                } else {
                    false
                }
            },
        )
        .map(|tok| tok.span())
    }

    pub fn eat_multi_symbol(&mut self, expected_symbol: &str) -> PResult<Span> {
        assert!(expected_symbol.chars().all(|c| SYMBOL_CHARS.contains(c)));
        assert!(expected_symbol.len() >= 2);
        let fork = self.fork_lexer();

        let mut token = self.next_token();
        let start = token.span().start_pos();
        for (i, symbol) in expected_symbol.chars().enumerate() {
            if i != 0 {
                token = self.next_token_incl_layout();
            }

            if let Token::Symbol(span) = token
                && self.db.input.inner().slice(span) == String::from_iter([symbol])
            {
            } else {
                let span = start.span_to(token.span().end_pos());
                self.recover_lexer_fork(&fork);
                return Err(self.expect(span, Expected::Literal(expected_symbol.to_string())));
            }
        }
        let span = start.span_to(token.span().end_pos());
        Ok(span)
    }
}
