use logos::{Logos};

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
pub enum LogosLexerToken {
    #[token("(")]
    ParenOpen,

    #[token(")")]
    ParenClose,

    #[token("{")]
    BracketOpen,

    #[token("}")]
    BracketClose,

    #[regex(r"[\p{Letter}\p{Mark}\p{Symbol}\p{Number}\p{Dash_Punctuation}\p{Connector_Punctuation}\p{Other_Punctuation}]+")]
    Identifier,

    #[token("\n")]
    Line,

    #[error]
    #[regex(r"[\p{Separator}\r]+", logos::skip)]
    Error,
}