use crate::input::Input;

/// Represents a certain range of a file. This is useful for marking the locations that certain tokens or errors occur.
/// The position and length are both in BYTES. The byte offsets provided should be valid.
#[derive(Debug, Clone)]
pub struct Span<'src> {
    pub position: usize,
    pub length: usize,
    pub source: Input<'src>,
}

impl<'src> Span<'src> {
    /// Creates a new span, given a file, starting position and the length that the span should be.
    pub fn from_length(source: Input<'src>, position: usize, length: usize) -> Self {
        Self {
            source,
            position,
            length,
        }
    }

    /// Creates a new span, given a file, starting position and end position.
    pub fn from_end(source: Input<'src>, position: usize, end: usize) -> Self {
        assert!(end >= position);
        Self {
            source,
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
        &self.source.full_input()[self.position..self.position + self.length]
    }

    /// Merge two spans.
    pub fn merge(&self, other: &Span) -> Self {
        // TODO: add unique ids to source files to make this comparison
        // assert!(self.source == other.source)

        Self::from_end(
            self.source,
            self.position.min(other.position),
            self.end().max(other.end()),
        )
    }
}
