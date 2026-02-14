use crate::span::Span;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum TokenType {
    CharClass,
    Keyword,
    Symbol,
    Slice,
    String,
    Number,
    Variable,
    Layout,
}

impl FromStr for TokenType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "charclass" => TokenType::CharClass,
            "keyword" => TokenType::Keyword,
            "symbol" => TokenType::Symbol,
            "slice" => TokenType::Slice,
            "string" => TokenType::String,
            "number" => TokenType::Number,
            "variable" => TokenType::Variable,
            "layout" => TokenType::Layout,
            _ => return Err(()),
        })
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenType::CharClass => "charclass",
                TokenType::Keyword => "keyword",
                TokenType::Symbol => "symbol",
                TokenType::Slice => "slice",
                TokenType::String => "string",
                TokenType::Number => "number",
                TokenType::Variable => "variable",
                TokenType::Layout => "layout",
            }
        )
    }
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
