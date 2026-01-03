use crate::error::ParseError;
use crate::error::error_label::ErrorLabel;
use prism_diags::{Annotation, AnnotationGroup, Diag};
use prism_input::pos::Pos;
use prism_input::span::Span;
use std::cmp::max;
use std::collections::{HashMap, HashSet};

/// Set error keeps track of the set of labels at the furthest position.
#[derive(Clone)]
pub struct SetError {
    pub span: Span,
    pub labels: HashSet<ErrorLabel>,
    pub explicit: bool,
}

impl ParseError for SetError {
    type L = ErrorLabel;

    fn new(start_pos: Pos) -> Self {
        Self {
            span: Span::new(start_pos, 0),
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
        assert_eq!(self.span.start_pos(), other.span.start_pos());
        for e in other.labels {
            self.labels.insert(e);
        }
        Self {
            span: Span::new(
                self.span.start_pos(),
                max(self.span.len(), other.span.len()),
            ),
            labels: self.labels,
            explicit: self.explicit || other.explicit,
        }
    }

    fn span(&self) -> Span {
        self.span
    }

    fn set_end(&mut self, end: Pos) {
        self.span = Span::new_with_end(self.span.start_pos(), end);
    }

    fn diag(&self) -> Diag {
        let mut labels_map: HashMap<Pos, Vec<_>> = HashMap::new();
        for l in self.labels.iter() {
            labels_map.entry(l.span().start_pos()).or_default().push(l);
        }

        Diag {
            title: "Parsing failed",
            id: "parser",
            groups: vec![AnnotationGroup {
                annotations: labels_map
                    .into_iter()
                    .map(|(start, labels)| Annotation {
                        span: start.span_to(start),
                        label: match labels[..] {
                            [] => unreachable!(),
                            [label] => format!("Expected: {}", label),
                            ref labels => format!(
                                "Expected one of: {}",
                                labels
                                    .iter()
                                    .map(|v| v.to_string())
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            ),
                        },
                    })
                    .collect(),
            }],
        }
    }
}
