use crate::parser::core::error::ParseError;
use crate::parser::core::span::Span;
use std::cmp::max;
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct SetError<L: Eq + Hash + Clone> {
    pub span: Span,
    pub labels: HashSet<L>,
}

impl<L: Eq + Hash + Clone> ParseError for SetError<L> {
    type L = L;

    fn new(span: Span) -> Self {
        Self {
            span,
            labels: HashSet::new(),
        }
    }

    fn add_label(&mut self, label: L) {
        if self.labels.is_empty() {
            self.labels.insert(label);
        }
    }

    fn merge(mut self, other: Self) -> Self {
        assert_eq!(self.span.start, other.span.start);
        for e in other.labels {
            self.labels.insert(e);
        }
        Self {
            span: Span::new(self.span.start, max(self.span.end, other.span.end)),
            labels: self.labels,
        }
    }
}
