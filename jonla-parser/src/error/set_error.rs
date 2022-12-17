use crate::error::ParseError;
use crate::core::span::Span;
use std::cmp::max;
use std::collections::HashSet;
use std::hash::Hash;

/// Set error keeps track of the set of labels at the furthest position.
#[derive(Clone, Debug)]
pub struct SetError<L: Eq + Hash + Clone> {
    pub span: Span,
    pub labels: HashSet<L>,
    pub explicit: bool,
}

impl<L: Eq + Hash + Clone> ParseError for SetError<L> {
    type L = L;

    fn new(span: Span) -> Self {
        Self {
            span,
            labels: HashSet::new(),
            explicit: false,
        }
    }

    fn add_label_explicit(&mut self, label: Self::L) {
        if !self.explicit {
            self.explicit = true;
            self.labels.clear();
        }
        self.labels.insert(label);
    }

    fn add_label_implicit(&mut self, label: Self::L) {
        if self.explicit {
            return;
        }
        self.labels.insert(label);
    }

    fn merge(mut self, other: Self) -> Self {
        assert_eq!(self.span.start, other.span.start);
        for e in other.labels {
            self.labels.insert(e);
        }
        Self {
            span: Span::new(self.span.start, max(self.span.end, other.span.end)),
            labels: self.labels,
            explicit: self.explicit || other.explicit,
        }
    }

    fn set_end(&mut self, end: usize) {
        self.span.end = end;
    }
}
