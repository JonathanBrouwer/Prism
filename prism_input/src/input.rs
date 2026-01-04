use crate::input_table::InputTable;
use crate::span::Span;
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
    pub fn from_span(span: Span, input: &InputTable) -> Self {
        Self(input.inner().slice(span).to_string().into())
    }

    pub fn as_str(&self, _input: &InputTable) -> &str {
        &self.0
    }

    pub fn parse_escaped_string(&self, _input: &InputTable) -> Self {
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
}
