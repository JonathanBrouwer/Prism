use crate::core::parsable::Parsable;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use std::borrow::Cow;

#[derive(Copy, Clone)]
pub enum Input<'grm> {
    Value(Span),
    Literal(EscapedString<'grm>),
}

impl<'grm> Input<'grm> {
    pub fn as_cow(&self, src: &'grm str) -> std::borrow::Cow<'grm, str> {
        match self {
            Self::Value(span) => std::borrow::Cow::Borrowed(&src[*span]),
            Self::Literal(s) => s.to_cow(),
            _ => panic!("Tried to get value of non-valued action result"),
        }
    }

    pub fn as_str(self, src: &'grm str) -> &'grm str {
        match self {
            Self::Value(span) => &src[span],
            Self::Literal(s) => match s.to_cow() {
                Cow::Borrowed(s) => s,
                Cow::Owned(_) => panic!("Tried to convert escaped literal to string"),
            },
        }
    }
}

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for Input<'grm> {}
