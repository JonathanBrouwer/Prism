use miette::{MietteError, SourceCode, SourceSpan, SpanContents};
use serde::{Deserialize, Serialize};
use crate::core_parser::source_file::SourceFile;

/// Represents a certain range of a file. This is useful for marking the locations that certain tokens or errors occur.
/// The position and length are both in BYTES. The byte offsets provided should be valid.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Span {
    pub position: usize,
    pub length: usize,
    pub source: SourceFile,
}

impl Span {
    /// Creates a new span, given a file, starting position and the length that the span should be.
    pub fn from_length(source: &SourceFile, position: usize, length: usize) -> Self {
        Self {
            source: source.clone(),
            position,
            length,
        }
    }

    /// Creates a new span, given a file, starting position and end position.
    pub fn from_end(source: &SourceFile, position: usize, end: usize) -> Self {
        assert!(end >= position);
        Self {
            source: source.clone(),
            position,
            length: end - position,
        }
    }

    pub fn end(&self) -> usize {
        self.position + self.length
    }

    /// Get a string from the source file, described by this span.
    /// ```
    /// // TODO
    /// ```
    pub fn as_str(&self) -> &str {
        &self.source.contents()[self.position..self.position + self.length]
    }

    /// Merge two spans.
    pub fn merge(&self, other: &Span) -> Self {
        // TODO: add unique ids to source files to make this comparison
        // assert!(self.source == other.source)

        Self::from_end(
            &self.source,
            self.position.min(other.position),
            self.end().max(other.end()),
        )
    }
}

impl SourceCode for Span {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        <str as SourceCode>::read_span(
            self.source.contents_for_display(),
            span,
            context_lines_before,
            context_lines_after,
        )
    }
}

impl From<Span> for SourceSpan {
    fn from(span: Span) -> Self {
        SourceSpan::from((span.position, span.length))
    }
}
