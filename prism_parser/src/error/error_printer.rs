use ariadne::{Color, Config, Label, LabelAttach, Report, ReportBuilder, ReportKind};
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
