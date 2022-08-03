use ariadne::{Color, Config, Label, LabelAttach, Report, ReportKind, Source};
use itertools::Itertools;
use crate::parser::core::error::FullError;
use crate::parser::core::span::Span;

#[derive(Eq, Hash, Clone, PartialEq)]
pub enum ErrorLabel<'grm> {
    Explicit(Span, &'grm str),
    Literal(Span, &'grm str),
}

impl ErrorLabel<'_> {
    fn span(&self) -> Span {
        match self {
            ErrorLabel::Explicit(s, _) => *s,
            ErrorLabel::Literal(s, _) => *s,
        }
    }
}

pub fn print_error(error: FullError<ErrorLabel>, input: &str) {
    let mut report = Report::build(ReportKind::Error, (), error.span.start)
        //Config
        .with_config(
            Config::default()
                .with_compact(true)
                .with_label_attach(LabelAttach::Start),
        )
        //Header
        .with_message("Parsing error")
        //Pointing label
        .with_label(
            Label::new(error.span.start..error.span.end)
                .with_message("This was the first character that was unparsable, starting at the marked position it could've been...")
                .with_color(Color::Red)
                .with_priority(1)
                .with_order(i32::MIN),
        );

    //Add labels
    for label in error.labels {
        let msg = match label {
            ErrorLabel::Explicit(_, s) => format!("{}", s),
            ErrorLabel::Literal(_, s) => format!("Literal '{}'", s),
        };
        report = report.with_label(Label::new(label.span().start..label.span().end).with_message(msg).with_order(-(label.span().start as i32)));
    }

    report
        .finish()
        .eprint(Source::from(input))
        .unwrap();
}