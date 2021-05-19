use logos::{Logos};

use std::fmt::Debug;

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy)]
pub enum LogosToken {
    #[regex(r"([\p{Letter}\p{Mark}\p{Symbol}\p{Number}\p{Dash_Punctuation}\p{Connector_Punctuation}\p{Other_Punctuation}]+)|[\p{Open_Punctuation}\p{Close_Punctuation}]")]
    Name,

    #[token("\n")]
    Line,

    #[error]
    #[regex(r"[\p{Separator}\r]+", logos::skip)]
    Error,
}
