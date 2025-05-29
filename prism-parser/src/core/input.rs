use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::parsable::parsed::{ArcExt, Parsed};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Eq, PartialEq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct Input(Arc<str>);

impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl Input {
    pub fn from_parsed(parsed: &Parsed) -> Self {
        parsed.value_ref::<Self>().clone()
    }

    pub fn to_parsed(self) -> Parsed {
        //TODO this is in an extra layer of `Arc` :c
        Arc::new(self).to_parsed()
    }

    pub fn from_span(span: Span, input: &InputTable) -> Self {
        Self(input.inner().slice(span).to_string().into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse_escaped_string(&self) -> Self {
        let mut result = String::new();
        let mut chars = self.0.chars();

        while let Some(c) = chars.next() {
            result.push(match c {
                '\\' => match chars.next().expect("Cannot escape end of str") {
                    'n' => '\n',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    _ => panic!("Invalid escape sequence"),
                },
                c => c,
            });
        }

        Self(result.into())
    }

    pub fn from_const(c: &'static str) -> Self {
        Self(c.to_string().into())
    }
}
