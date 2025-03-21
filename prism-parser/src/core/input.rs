use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use crate::parsable::ParseResult;
use std::borrow::Cow;

#[derive(Copy, Clone)]
pub enum Input<'grm> {
    Value(Span),
    Literal(EscapedString<'grm>),
}

impl<'grm> Input<'grm> {
    pub fn as_cow(&self, src: &InputTable<'grm>) -> Cow<'grm, str> {
        match self {
            Self::Value(span) => Cow::Borrowed(&src[*span]),
            Self::Literal(s) => s.to_cow(),
        }
    }

    pub fn as_str(self, src: &InputTable<'grm>) -> &'grm str {
        match self {
            Self::Value(span) => &src[span],
            Self::Literal(s) => match s.to_cow() {
                Cow::Borrowed(s) => s,
                Cow::Owned(_) => panic!("Tried to convert escaped literal to string"),
            },
        }
    }
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for Input<'grm> {}
