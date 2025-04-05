use crate::core::span::Span;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Tokens {
    Single(Token),
    Multi(Vec<Arc<Tokens>>),
}

#[derive(Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TokenType {
    CharClass,
    Keyword,
    Symbol,
    Slice,
}

impl Tokens {
    pub fn to_vec(&self) -> Vec<Token> {
        fn insert_tokens(tokens: &Tokens, v: &mut Vec<Token>) {
            match tokens {
                Tokens::Single(token) => v.push(token.clone()),
                Tokens::Multi(tokens) => {
                    for token in tokens {
                        insert_tokens(token, v);
                    }
                }
            }
        }

        let mut tokens = vec![];
        insert_tokens(self, &mut tokens);
        tokens
    }
}
