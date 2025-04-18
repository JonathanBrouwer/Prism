use crate::core::pos::Pos;
use crate::core::span::Span;
use crate::error::ParseError;
use crate::error::error_printer::{ErrorLabel, base_report};
use ariadne::{Label, Report};
use std::cmp::max;
use std::collections::{BTreeMap, HashSet};

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

    fn report(&self) -> Report<'static, Span> {
        let mut report = base_report(self.span);

        let mut labels_map: BTreeMap<Pos, Vec<_>> = BTreeMap::new();
        for l in self.labels.iter() {
            labels_map.entry(l.span().start_pos()).or_default().push(l);
        }

        //Add labels
        for (start, labels) in labels_map {
            report = report.with_label(
                Label::new(start.span_to(start))
                    .with_message(format!(
                        "Tried parsing {}",
                        labels
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(" / ")
                    ))
                    .with_order(-(start.idx_in_file() as i32)),
            );
        }

        report.finish()
    }
}
