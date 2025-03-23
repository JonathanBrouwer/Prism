use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use crate::parsable::ParseResult;
use std::borrow::Cow;

#[derive(Copy, Clone)]
pub enum Input<'arn> {
    Value(Span),
    Literal(EscapedString<'arn>),
}

impl<'arn> Input<'arn> {
    pub fn as_cow(&self, src: &InputTable<'arn>) -> Cow<'arn, str> {
        match self {
            Self::Value(span) => Cow::Borrowed(src.slice(*span)),
            Self::Literal(s) => s.to_cow(),
        }
    }

    pub fn as_str(self, src: &InputTable<'arn>) -> &'arn str {
        match self {
            Self::Value(span) => src.slice(span),
            Self::Literal(s) => match s.to_cow() {
                Cow::Borrowed(s) => s,
                Cow::Owned(_) => panic!("Tried to convert escaped literal to string"),
            },
        }
    }
}

impl ParseResult for Input<'_> {}
