use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use miette::{Diagnostic, GraphicalReportHandler, LabeledSpan, Severity, SourceCode};
use crate::peg::input::Input;

pub type Span = (usize, usize);

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct JError<I: Input> {
    pub errors: Vec<JErrorEntry>,
    pub pos: I,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum JErrorEntry {
    UnexpectedEOF(Span, ),
    UnexpectedChar(Span, char),
    UnexpectedStr(Span, &'static str),
    UnexpectedString(Span, String),
}

impl JErrorEntry {
    pub fn message(&self) -> String {
        match self {
            JErrorEntry::UnexpectedEOF(_) => "Parsing error",
            JErrorEntry::UnexpectedChar(_, _) => "Parsing error",
            JErrorEntry::UnexpectedStr(_, _) => "Parsing error",
            JErrorEntry::UnexpectedString(_, _) => "Parsing error",
        }.to_string()
    }
    pub fn severity(&self) -> Severity {
        Severity::Error
    }
    pub fn labels(&self) -> Vec<JErrorLabel> {
        match self {
            JErrorEntry::UnexpectedEOF(span) => vec![JErrorLabel{ msg: Some(format!("Expected more input, but found end of file.")), span: *span }],
            JErrorEntry::UnexpectedChar(span, msg) => vec![JErrorLabel{ msg: Some(format!("Expected {}, but found end of file.", msg)), span: *span }],
            JErrorEntry::UnexpectedStr(span, msg) => vec![JErrorLabel{ msg: Some(format!("Expected {}, but found end of file.", msg)), span: *span }],
            JErrorEntry::UnexpectedString(span, msg) => vec![JErrorLabel{ msg: Some(format!("Expected {}, but found end of file.", msg)), span: *span }],
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct JErrorLabel {
    pub msg: Option<String>,
    pub span: Span,
}



impl<I: Input> JError<I> {
    pub(crate) fn parse_error_combine_opt2(e1: Option<JError<I>>, e2: Option<JError<I>>) -> Option<JError<I>> {
        match (e1, e2) {
            (Some(e1), Some(e2)) => Some(e1.combine_or(e2)),
            (Some(e1), None) => Some(e1),
            (None, Some(e2)) => Some(e2),
            (None, None) => None,
        }
    }

    pub(crate) fn parse_error_combine_opt1(e1: JError<I>, e2: Option<JError<I>>) -> JError<I> {
        match e2 {
            Some(e2) => e1.combine_or(e2),
            None => e1
        }
    }

    pub fn combine_or(mut self, mut other: Self) -> Self {
        match self.pos.pos().cmp(&other.pos.pos()) {
            Ordering::Less => other,
            Ordering::Greater => self,
            Ordering::Equal => {
                self.errors.append(&mut other.errors);
                self
            }
        }
    }
}

impl<I: Input> Display for JError<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut src = self.pos.src_str().to_string();
        src += " "; //Allow to point to EOF
        for error in &self.errors {
            let diag = ParseDiagnostic { src: &src, msg: error.message(), severity: error.severity(), labels: error.labels() };
            let mut s = String::new();
            GraphicalReportHandler::new()
                .with_links(true)
                .render_report(&mut s, &diag)
                .unwrap();
            write!(f, "{}", s)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ParseDiagnostic<'a> {
    pub(crate) src: &'a str,
    pub(crate) msg: String,
    pub(crate) severity: Severity,
    pub(crate) labels: Vec<JErrorLabel>
}

impl<'a> Error for ParseDiagnostic<'a> {}

impl<'a> Display for ParseDiagnostic<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl<'b> Diagnostic for ParseDiagnostic<'b> {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        None
    }

    fn severity(&self) -> Option<Severity> {
        Some(self.severity)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        None
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        None
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.src)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        Some(Box::new(self.labels.iter().map(|l| LabeledSpan::new(l.msg.clone(), l.span.0, l.span.1 - l.span.0))))
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        None
    }
}
