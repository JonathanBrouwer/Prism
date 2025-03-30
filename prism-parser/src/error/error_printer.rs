use crate::core::input::Input;
use crate::core::input_table::{InputTable, InputTableIndex};
use crate::core::span::Span;
use crate::grammar::identifier::Identifier;
use ariadne::{Color, Config, Label, LabelAttach, Report, ReportBuilder, ReportKind};

#[derive(Eq, Hash, Clone, PartialEq)]
pub enum ErrorLabel {
    Explicit(Span, String),
    Literal(Span, String),
    Debug(Span, String),
}

impl ErrorLabel {
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

    pub fn to_string(&self, input: &InputTable) -> String {
        match self {
            ErrorLabel::Explicit(_, s) => s.to_string(),
            ErrorLabel::Literal(_, s) => s.to_string(),
            ErrorLabel::Debug(_, s) => format!("[{}]", s.to_string()),
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
