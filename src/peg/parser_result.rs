use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use miette::{Diagnostic, GraphicalReportHandler, LabeledSpan, Severity, SourceCode};
use crate::peg::input::Input;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ParseSuccess<I: Input, O> {
    pub result: O,
    pub best_error: Option<ParseError<I>>,
    pub pos: I,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ParseError<I: Input> {
    pub errors: Vec<ParseErrorEntry>,
    pub pos: I,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ParseErrorEntry {
    pub msg: String,
    pub severity: Severity,
    pub labels: Vec<ParseErrorLabel>
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ParseErrorLabel {
    pub msg: String,
    pub at: usize,
}

impl<I: Input> ParseError<I> {
    pub(crate) fn parse_error_combine_opt2(e1: Option<ParseError<I>>, e2: Option<ParseError<I>>) -> Option<ParseError<I>> {
        match (e1, e2) {
            (Some(e1), Some(e2)) => Some(e1.combine_or(e2)),
            (Some(e1), None) => Some(e1),
            (None, Some(e2)) => Some(e2),
            (None, None) => None,
        }
    }

    pub(crate) fn parse_error_combine_opt1(e1: ParseError<I>, e2: Option<ParseError<I>>) -> ParseError<I> {
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

impl<I: Input> Display for ParseError<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut src = self.pos.src_str().to_string();
        src += " "; //Allow to point to EOF
        for error in &self.errors {
            let diag = ParseDiagnostic { src: &src, msg: error.msg.clone(), severity: error.severity, labels: error.labels.clone() };
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
pub struct ParseDiagnostic<'a> {
    pub(crate) src: &'a str,
    pub(crate) msg: String,
    pub(crate) severity: Severity,
    pub(crate) labels: Vec<ParseErrorLabel>
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
        Some(Box::new(self.labels.iter().map(|l| LabeledSpan::new(Some(l.msg.clone()), l.at, 1))))
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        None
    }
}
