use crate::parser::core::error::set_error::SetError;
use crate::parser::core::error::tree_error::TreeError;
use crate::parser::core::span::Span;
use ariadne::{Color, Config, Label, LabelAttach, Report, ReportBuilder, ReportKind, Source};
use std::fmt::{Display, Formatter};
use std::ops::Range;
use itertools::Itertools;

#[derive(Eq, Hash, Clone, PartialEq)]
pub enum ErrorLabel<'grm> {
    Explicit(Span, &'grm str),
    Literal(Span, &'grm str),
    Debug(Span, &'grm str),
}

impl ErrorLabel<'_> {
    fn span(&self) -> Span {
        match self {
            ErrorLabel::Explicit(s, _) => *s,
            ErrorLabel::Literal(s, _) => *s,
            ErrorLabel::Debug(s, _) => *s,
        }
    }

    fn is_debug(&self) -> bool {
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

pub fn print_base(span: Span, filename: &str) -> ReportBuilder<(&str, Range<usize>)> {
    Report::build::<&str>(ReportKind::Error, filename, span.start)
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
            Label::new((filename, span.start..span.end))
                .with_message("This was the first character that was unparsable, starting at the marked position it could've been...")
                .with_color(Color::Red)
                .with_priority(1)
                .with_order(i32::MIN),
        )
}

pub fn print_set_error(error: SetError<ErrorLabel>, filename: &str, input: &str, enable_debug: bool) {
    let mut report = print_base(error.span, filename);

    //Add labels
    for (start, labels) in error.labels.into_iter().filter(|l| enable_debug || !l.is_debug()).into_group_map_by(|l| l.span().start).into_iter() {
        report = report.with_label(
            Label::new((filename, start..error.span.end))
                .with_message(format!("{}", labels.into_iter().format(" / ")))
                .with_order(-(start as i32)),
        );
    }

    report.finish().eprint((filename, Source::from(input))).unwrap();
}

pub fn print_tree_error(error: TreeError<ErrorLabel>, filename: &str, input: &str) {
    let mut report = print_base(error.span, filename);

    //Add labels
    for path in error.labels.into_paths() {
        if path.is_empty() { continue; }
        let label = &path[0];

        report = report.with_label(
            Label::new((filename, label.span().start..label.span().end))
                .with_message(format!("{}", path.iter().format(" <- ")))
                .with_order(-(label.span().start as i32)),
        );
    }

    report.finish().eprint((filename, Source::from(input))).unwrap();
}
