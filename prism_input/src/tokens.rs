use crate::span::Span;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Tokens(pub Vec<Token>);

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
    Comment,
}
