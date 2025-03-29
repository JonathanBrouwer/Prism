use crate::META_GRAMMAR_STR;
use crate::core::input_table::{InputTable, META_INPUT_INDEX};
use crate::core::pos::Pos;
use crate::core::span::Span;
use crate::parsable::ParseResult;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::Chars;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Deserialize)]
pub struct Input {
    span: Span,
    has_escapes: bool,
}

impl<'arn> Serialize for Input {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = Serializer::serialize_struct(serializer, "Input", 2)?;
        state.serialize_field("span", &self.span.unsafe_set_file(META_INPUT_INDEX))?;
        state.serialize_field("has_escapes", &self.has_escapes)?;
        state.end()
    }
}

// impl<'de> Deserialize<'de> for Input {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let s: &'de str = Deserialize::deserialize(deserializer)?;
//         let meta_str_start = META_GRAMMAR_STR.find(s).unwrap();
//         Ok(Input {
//             span: Span::new(Pos::start_of(META_INPUT_INDEX) + meta_str_start, s.len()),
//             has_escapes: true,
//         })
//     }
// }

impl<'arn> Input {
    pub fn from_span(span: Span) -> Self {
        Self {
            span,
            has_escapes: false,
        }
    }

    pub fn to_string(self, input: &InputTable<'arn>) -> String {
        self.chars(input).collect()
    }

    pub fn chars(self, input: &InputTable<'arn>) -> impl Iterator<Item = char> + use<'arn> {
        struct EscapedStringIter<'arn>(Chars<'arn>, bool);
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

    pub fn as_str(self, input: &InputTable<'arn>) -> &'arn str {
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
