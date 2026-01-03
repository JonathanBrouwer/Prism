use prism_input::span::Span;
use std::fmt::{Display, Formatter};

#[derive(Eq, Hash, Clone, PartialEq)]
pub enum ErrorLabel {
    Explicit(Span, String),
    Literal(Span, String),
}

impl ErrorLabel {
    pub(crate) fn span(&self) -> Span {
        match self {
            ErrorLabel::Explicit(s, _) => *s,
            ErrorLabel::Literal(s, _) => *s,
        }
    }
}

impl Display for ErrorLabel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorLabel::Explicit(_, s) => write!(f, "{s}"),
            ErrorLabel::Literal(_, s) => write!(f, "{s}"),
        }
    }
}
