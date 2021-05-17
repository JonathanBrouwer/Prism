use logos::Span;
use crate::lexer::lexer::{LexerToken, LexerTokenType};


pub struct JonlaParser<'a> {
    pub source: &'a str,
    pub tokens: Vec<LexerToken<'a>>,

    pub cursor: usize,
}

impl<'a> JonlaParser<'a> {
    pub fn new(source: &'a str, tokens: Vec<LexerToken<'a>>) -> JonlaParser<'a> {
        JonlaParser { source, tokens, cursor: 0 }
    }

    pub fn peek(&self) -> Option<&LexerToken<'a>> {
        if self.cursor >= self.tokens.len() {
            None
        } else {
            Some(&self.tokens[self.cursor])
        }
    }

    pub fn next(&mut self) -> Option<&LexerToken<'a>> {
        if self.cursor >= self.tokens.len() {
            None
        } else {
            self.cursor += 1;
            Some(&self.tokens[self.cursor - 1])
        }
    }

    pub fn expect(&mut self, expected: LexerTokenType) -> Result<&LexerToken<'a>, String> {
        let next = self.next();
        if let Some(token) = next {
            if token.token == expected {
                Ok(token)
            } else {
                Err(format!("Expected {:?}, but got {:?}", expected, token))
            }
        } else {
            Err(format!("Expected {:?}, but reached end of file.", expected))
        }
    }

    pub fn expect_identifier(&mut self) -> Result<&'a str, String> {
        let next = self.next();
        if let Some(LexerToken{span: _, token: LexerTokenType::Identifier(id)}) = next {
            Ok(id)
        } else {
            Err(format!("Expected an identifier, but got {:?}.", next))
        }
    }

    pub fn expect_keyword(&mut self, keyword: &str) -> Result<(), String> {
        let id = self.expect_identifier()?;
        if id == keyword {
            Ok(())
        } else {
            Err(format!("Expected the keyword {:?}, but got {:?}.", keyword, id))
        }
    }
}



