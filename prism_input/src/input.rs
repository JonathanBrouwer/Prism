use crate::input_table::InputTable;
use crate::span::Span;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Eq, PartialEq, Hash, Clone, Debug, Deserialize, Serialize)]
pub struct Input {
    s: Arc<str>,
    escaped: bool,
}

impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.s)
    }
}

impl Input {
    pub fn from_span(span: Span, input: &InputTable) -> Self {
        Self {
            s: input.inner().slice(span).to_string().into(),
            escaped: false,
        }
    }

    pub fn as_str(&self, _input: &InputTable) -> Cow<'_, str> {
        if self.escaped {
            let mut result = String::new();
            let mut chars = self.s.chars();

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
            Cow::Owned(result)
        } else {
            Cow::Borrowed(&self.s)
        }
    }

    pub fn parse_escaped_string(&self, _input: &InputTable) -> Self {
        assert!(!self.escaped);
        if !self.s.contains("\\") {
            return self.clone();
        }
        Self {
            s: self.s.clone(),
            escaped: true,
        }
    }
}
