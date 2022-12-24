use crate::core::pos::Pos;
use crate::core::span::Span;
use crate::error::set_error::SetError;
use crate::error::tree_error::TreeError;
use crate::grammar::grammar::EscapedString;
use ariadne::{Color, Config, Label, LabelAttach, Report, ReportBuilder, ReportKind, Source};
use itertools::Itertools;
use std::fmt::{Display, Formatter};

#[derive(Eq, Hash, Clone, PartialEq)]
pub enum ErrorLabel<'grm> {
    Explicit(Span, EscapedString<'grm>),
    Literal(Span, EscapedString<'grm>),
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

pub fn print_base(span: Span) -> ReportBuilder<Span> {
    Report::build(ReportKind::Error, (), span.start.into())
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
            Label::new(span)
                .with_message(match span.end - span.start {
                    0 => "Failed to parse at this location (but recovered immediately), errors are marked at attempted parse positions.",
                    1 => "This character was unparsable, errors are marked at attempted parse positions.",
                    _ => "These characters were unparsable, errors are marked at attempted parse positions.",
                })
                .with_color(Color::Red)
                .with_priority(1)
                .with_order(i32::MIN),
        )
}

pub fn print_set_error(error: SetError<ErrorLabel>, input: &str, enable_debug: bool) {
    let mut report = print_base(error.span);

    //Add labels
    for (start, labels) in error
        .labels
        .into_iter()
        .filter(|l| enable_debug || !l.is_debug())
        .into_group_map_by(|l| l.span().start)
        .into_iter()
    {
        report = report.with_label(
            Label::new(start.span_to(start))
                .with_message(format!("Expected {}", labels.into_iter().format(" / ")))
                .with_order(-(<Pos as Into<usize>>::into(start) as i32)),
        );
    }

    report.finish().eprint(Source::from(input)).unwrap();
}

pub fn print_tree_error(error: TreeError<ErrorLabel>, input: &str, enable_debug: bool) {
    let mut report: ReportBuilder<Span> = print_base(error.span);

    //Add labels
    for path in error.labels.into_paths() {
        let path = path
            .iter()
            .filter(|l| enable_debug || !l.is_debug())
            .collect_vec();
        if path.is_empty() {
            continue;
        }
        let label = &path[0];

        report = report.with_label(
            Label::new(label.span())
                .with_message(format!("{}", path.iter().format(" <- ")))
                .with_order(-(<Pos as Into<usize>>::into(label.span().start) as i32)),
        );
    }

    report.finish().eprint(Source::from(input)).unwrap();
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
