use crate::core::input_table::InputTable;
use crate::core::pos::Pos;
use crate::core::span::Span;
use crate::error::ParseError;
use crate::error::error_printer::{ErrorLabel, base_report};
use ariadne::{Label, Report};
use std::collections::{BTreeMap, HashSet};

/// Set error keeps track of the set of labels at the furthest position.
#[derive(Clone)]
pub struct SetError<'arn> {
    pub pos: Pos,
    pub labels: HashSet<ErrorLabel<'arn>>,
    pub explicit: bool,
}

impl<'arn> ParseError for SetError<'arn> {
    type L = ErrorLabel<'arn>;

    fn new(span: Pos) -> Self {
        Self {
            pos: span,
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
        assert_eq!(self.pos, other.pos);
        for e in other.labels {
            self.labels.insert(e);
        }
        Self {
            pos: self.pos,
            labels: self.labels,
            explicit: self.explicit || other.explicit,
        }
    }

    fn report(&self, enable_debug: bool, input: &InputTable) -> Report<'static, Span> {
        let mut report = base_report(self.pos.span_to(self.pos));

        let mut labels_map: BTreeMap<Pos, Vec<_>> = BTreeMap::new();
        for l in self.labels.iter().filter(|l| enable_debug || !l.is_debug()) {
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
                            .map(|v| v.to_string(input))
                            .collect::<Vec<_>>()
                            .join(" / ")
                    ))
                    .with_order(-(start.idx_in_file() as i32)),
            );
        }

        report.finish()
    }
}
