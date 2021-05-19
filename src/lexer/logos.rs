use logos::{Logos};

use std::fmt::Debug;

const REGEX_IDENTIFIER: &str = r"[\p{Letter}\p{Mark}\p{Symbol}\p{Number}\p{Dash_Punctuation}\p{Connector_Punctuation}\p{Other_Punctuation}]";
const REGEX_CONTROL: &str = r"\p{Open_Punctuation}\p{Close_Punctuation}";

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
