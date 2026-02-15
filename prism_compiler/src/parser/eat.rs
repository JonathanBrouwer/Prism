use crate::parser::ParserPrismEnv;
use crate::parser::expect::{Expected, PResult};
use crate::parser::lexer::Token;
use prism_input::span::Span;

impl<'a> ParserPrismEnv<'a> {
    pub fn eat_token(
        &mut self,
        expected: Expected,
        f: impl Fn(Token, &mut Self) -> bool,
    ) -> PResult<Token> {
        let token = self.token();
        if f(token, self) {
            self.next_token();
            Ok(token)
        } else {
            Err(self.expect(self.token().span(), expected))
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

    pub fn eat_paren_open(&mut self, open: &str) -> PResult<()> {
        self.eat_token(Expected::Literal(open.to_string()), |token, env| {
            if let Token::OpenParen(span) = token
                && env.db.input.inner().slice(span) == open
            {
                true
            } else {
                false
            }
        })
        .map(|_| ())
    }

    pub fn eat_paren_close(&mut self, close: &str) -> PResult<()> {
        self.eat_token(Expected::Literal(close.to_string()), |token, env| {
            if let Token::CloseParen(span) = token
                && env.db.input.inner().slice(span) == close
            {
                true
            } else {
                false
            }
        })
        .map(|_| ())
    }

    pub fn eat_identifier(&mut self) -> PResult<Span> {
        self.eat_token(Expected::Rule("identifier".to_string()), |token, _env| {
            matches!(token, Token::Identifier { .. })
        })
        .map(|tok| tok.span())
    }

    pub fn eat_symbol(&mut self, expected_symbol: char) -> PResult<Span> {
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
}
