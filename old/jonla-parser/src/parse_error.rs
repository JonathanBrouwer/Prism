use crate::character_class::CharacterClass;
use crate::span::Span;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// A parsing error represents a single error that occurred during parsing.
/// The parsing error occurs at a certain position in a file, represented by the span.
/// The parsing error consists of multiple `ParseErrorSub`, which each represent a single thing that went wrong at this position.
#[derive(Debug, Clone, Error)]
#[error("A parse error occured!")]
pub struct PEGParseError<'src> {
    pub span: Span<'src>,
    pub expected: Vec<Expect>,
    pub fail_left_rec: bool,
    pub fail_loop: bool,
}

impl<'src> PEGParseError<'src> {
    pub fn expect(span: Span<'src>, expect: Expect) -> Self {
        PEGParseError {
            span,
            expected: vec![expect],
            fail_left_rec: false,
            fail_loop: false,
        }
    }

    pub fn fail_left_recursion(span: Span<'src>) -> Self {
        PEGParseError {
            span,
            expected: vec![],
            fail_left_rec: true,
            fail_loop: false,
        }
    }

    pub fn fail_loop(span: Span<'src>) -> Self {
        PEGParseError {
            span,
            expected: vec![],
            fail_left_rec: false,
            fail_loop: true,
        }
    }
}

/// Represents a single thing that went wrong at this position.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Expect {
    /// Expect a character from a certain char class to be there, but it was not.
    ExpectCharClass(CharacterClass),

    /// Expect a certain string (keyword) to be there, but it was not.
    ExpectString(String),

    /// Expect a certain sort
    ExpectSort(String),

    /// This happens when not the entire input was parsed, but also no errors occurred during parsing.
    NotEntireInput(),
}

impl Display for Expect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expect::ExpectCharClass(cc) => {
                write!(f, "{}", cc)
            }
            Expect::ExpectString(s) => {
                write!(f, "\'{}\'", s)
            }
            Expect::ExpectSort(s) => {
                write!(f, "{}", s)
            }
            Expect::NotEntireInput() => {
                write!(f, "more input")
            }
        }
    }
}

impl<'src> PEGParseError<'src> {
    /// Combine multiple parse errors. When one has precedence over
    /// another, the highest precedence error is kept and the other
    /// is discarded.
    ///
    /// Highest precedence is defined as furthest starting position for now. This might be changed later.
    pub fn combine(mut self, mut other: PEGParseError<'src>) -> PEGParseError<'src> {
        assert_eq!(self.span.source.file_path(), other.span.source.file_path());

        //Compare the starting positions of the span
        match self.span.position.cmp(&other.span.position) {
            Ordering::Less => other,
            Ordering::Greater => self,
            Ordering::Equal => {
                //The span is extended such that the longest one is kept.
                self.span.length = self.span.length.max(other.span.length);
                //Merge the expected tokens
                self.expected.append(&mut other.expected);
                //Left recursion
                self.fail_left_rec |= other.fail_left_rec;

                self
            }
        }
    }

    /// A helper that combines optional parse errors, and returns an optional parse error if either exists.
    /// If both exist, use `ParseError::combine` to combine the errors.
    pub fn combine_option_parse_error(
        a: Option<PEGParseError<'src>>,
        b: Option<PEGParseError<'src>>,
    ) -> Option<PEGParseError<'src>> {
        match (a, b) {
            (None, None) => None,
            (None, Some(e)) => Some(e),
            (Some(e), None) => Some(e),
            (Some(e1), Some(e2)) => Some(e1.combine(e2)),
        }
    }
}
