use crate::parser::core::error::set_error::SetError;
use crate::parser::core::error::tree_error::TreeError;
use crate::parser::core::span::Span;
use ariadne::{Color, Config, Label, LabelAttach, Report, ReportKind, Source};
use std::fmt::{Display, Formatter};

#[derive(Eq, Hash, Clone, PartialEq)]
pub enum ErrorLabel<'grm> {
    Explicit(Span, &'grm str),
    Literal(Span, &'grm str),
    // Debug(Span, &'grm str),
}

impl ErrorLabel<'_> {
    fn span(&self) -> Span {
        match self {
            ErrorLabel::Explicit(s, _) => *s,
            ErrorLabel::Literal(s, _) => *s,
        }
    }
}

impl Display for ErrorLabel<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorLabel::Explicit(_, s) => write!(f, "{}", s),
            ErrorLabel::Literal(_, s) => write!(f, "'{}'", s),
        }
    }
}

pub fn print_set_error(error: SetError<ErrorLabel>, input: &str) {
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
        report = report.with_label(
            Label::new(label.span().start..label.span().end)
                .with_message(format!("{}", label))
                .with_order(-(label.span().start as i32)),
        );
    }

    report.finish().eprint(Source::from(input)).unwrap();
}

pub fn print_tree_error(error: TreeError<ErrorLabel>, input: &str) {
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
    for path in error.labels.into_paths() {
        if path.is_empty() { continue; }
        let label = &path[0];

        report = report.with_label(
            Label::new(label.span().start..label.span().end)
                .with_message(format!("{}", label))
                .with_order(-(label.span().start as i32)),
        );
    }

    report.finish().eprint(Source::from(input)).unwrap();
}
