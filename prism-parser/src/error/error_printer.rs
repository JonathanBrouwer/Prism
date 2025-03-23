use crate::core::input_table::InputTableIndex;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use ariadne::{Color, Config, Label, LabelAttach, Report, ReportBuilder, ReportKind};
use std::fmt::{Display, Formatter};

#[derive(Eq, Hash, Clone, PartialEq)]
pub enum ErrorLabel<'arn> {
    Explicit(Span, EscapedString<'arn>),
    Literal(Span, EscapedString<'arn>),
    Debug(Span, &'arn str),
    FromConstruct(Span, String),
}

impl ErrorLabel<'_> {
    pub(crate) fn span(&self) -> Span {
        match self {
            ErrorLabel::Explicit(s, _) => *s,
            ErrorLabel::Literal(s, _) => *s,
            ErrorLabel::Debug(s, _) => *s,
            ErrorLabel::FromConstruct(s, _) => *s,
        }
    }

    pub(crate) fn is_debug(&self) -> bool {
        match self {
            ErrorLabel::Explicit(_, _) => false,
            ErrorLabel::Literal(_, _) => false,
            ErrorLabel::Debug(_, _) => true,
            ErrorLabel::FromConstruct(_, _) => false,
        }
    }
}

impl Display for ErrorLabel<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorLabel::Explicit(_, s) => write!(f, "{s}"),
            ErrorLabel::Literal(_, s) => write!(f, "'{s}'"),
            ErrorLabel::Debug(_, s) => write!(f, "[{s}]"),
            ErrorLabel::FromConstruct(_, s) => write!(f, "{s}"),
        }
    }
}

pub fn base_report(span: Span) -> ReportBuilder<'static, Span> {
    Report::build(ReportKind::Error, span)
        //Config
        .with_config(Config::default().with_label_attach(LabelAttach::Start))
        //Header
        .with_message("Parsing error")
        //Pointing label
        .with_label(
            Label::new(span)
                .with_message(match span.len() {
                    0 => "Failed to parse at this location",
                    1 => "This character was unparsable",
                    _ => "These characters were unparsable",
                })
                .with_color(Color::Red)
                .with_priority(1)
                .with_order(i32::MIN),
        )
}

impl ariadne::Span for Span {
    type SourceId = InputTableIndex;

    fn source(&self) -> &Self::SourceId {
        self.start_pos_ref().file_ref()
    }

    fn start(&self) -> usize {
        self.start_pos().idx_in_file()
    }

    fn end(&self) -> usize {
        self.end_pos().idx_in_file()
    }
}
