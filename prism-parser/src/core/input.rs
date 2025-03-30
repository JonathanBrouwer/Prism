use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::parsable::ParseResult;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::Chars;

#[derive(Copy, Clone, Debug)]
pub struct Input {
    span: Span,
    has_escapes: bool,
}

impl<'arn> Serialize for Input {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        assert!(self.has_escapes);
        self.span.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Input {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Input {
            span: Span::deserialize(deserializer)?,
            has_escapes: true,
        })
    }
}

impl Input {
    pub fn from_span(span: Span) -> Self {
        Self {
            span,
            has_escapes: false,
        }
    }

    pub fn span(self) -> Span {
        self.span
    }

    pub fn to_string(self, input: &InputTable) -> String {
        self.chars(input).collect()
    }

    pub fn chars<'a>(self, input: &'a InputTable) -> impl Iterator<Item = char> + use<'a> {
        struct EscapedStringIter<'a>(Chars<'a>, bool);
        impl Iterator for EscapedStringIter<'_> {
            type Item = char;

            fn next(&mut self) -> Option<Self::Item> {
                Some(match self.0.next()? {
                    '\\' if self.1 => match self.0.next()? {
                        'n' => '\n',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        '\'' => '\'',
                        _ => panic!("Invalid escape sequence"),
                    },
                    c => c,
                })
            }
        }

        EscapedStringIter(input.slice(self.span).chars(), self.has_escapes)
    }

    pub fn as_str<'a>(self, input: &InputTable<'a>) -> &'a str {
        let slice = input.slice(self.span);
        assert!(!self.has_escapes || !slice.contains('\\'));
        slice
    }

    pub fn parse_escaped_string(self) -> Self {
        assert!(!self.has_escapes);
        Self {
            has_escapes: true,
            ..self
        }
    }
}

impl ParseResult for Input {}
