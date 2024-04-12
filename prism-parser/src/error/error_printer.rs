use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use ariadne::{Color, Config, Label, LabelAttach, Report, ReportBuilder, ReportKind};
use std::fmt::{Display, Formatter};

#[derive(Eq, Hash, Clone, PartialEq)]
pub enum ErrorLabel<'grm> {
    Explicit(Span, EscapedString<'grm>),
    Literal(Span, EscapedString<'grm>),
    Debug(Span, &'grm str),
}

impl ErrorLabel<'_> {
    pub(crate) fn span(&self) -> Span {
        match self {
            ErrorLabel::Explicit(s, _) => *s,
            ErrorLabel::Literal(s, _) => *s,
            ErrorLabel::Debug(s, _) => *s,
        }
    }

    pub(crate) fn is_debug(&self) -> bool {
        match self {
            ErrorLabel::Explicit(_, _) => false,
            ErrorLabel::Literal(_, _) => false,
            ErrorLabel::Debug(_, _) => true,
        }
    }
}

impl Display for ErrorLabel<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorLabel::Explicit(_, s) => write!(f, "{}", s),
            ErrorLabel::Literal(_, s) => write!(f, "'{}'", s),
            ErrorLabel::Debug(_, s) => write!(f, "[{}]", s),
        }
    }
}

pub fn base_report(span: Span) -> ReportBuilder<'static, Span> {
    Report::build(ReportKind::Error, (), 0)
        //Config
        .with_config(
            Config::default()
                .with_label_attach(LabelAttach::Start),
        )
        //Header
        .with_message("Parsing error")
        //Pointing label
        .with_label(
            Label::new(span)
                .with_message(match span.end - span.start {
                    0 => "Failed to parse at this location (but recovered immediately)",
                    1 => "This character was unparsable",
                    _ => "These characters were unparsable",
                })
                .with_color(Color::Red)
                .with_priority(1)
                .with_order(i32::MIN),
        )
}

impl ariadne::Span for Span {
    type SourceId = ();

    fn source(&self) -> &Self::SourceId {
        &()
    }

    fn start(&self) -> usize {
        self.start.into()
    }

    fn end(&self) -> usize {
        self.end.into()
    }
}
