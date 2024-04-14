use crate::core::pos::Pos;
use crate::core::span::Span;
use crate::error::error_printer::{base_report, ErrorLabel};
use crate::error::ParseError;
use ariadne::{Label, Report};
use itertools::Itertools;
use std::cmp::max;
use std::collections::HashSet;

/// Set error keeps track of the set of labels at the furthest position.
#[derive(Clone)]
pub struct SetError<'grm> {
    pub span: Span,
    pub labels: HashSet<ErrorLabel<'grm>>,
    pub explicit: bool,
}

impl<'grm> ParseError for SetError<'grm> {
    type L = ErrorLabel<'grm>;

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

    fn set_end(&mut self, end: Pos) {
        self.span.end = end;
    }

    fn report(&self, enable_debug: bool) -> Report<'static, Span> {
        let mut report = base_report(self.span);

        //Add labels
        for (start, labels) in self
            .labels
            .iter()
            .filter(|l| enable_debug || !l.is_debug())
            .into_group_map_by(|l| l.span().start)
            .into_iter()
        {
            report = report.with_label(
                Label::new(start.span_to(start))
                    .with_message(format!("Expected {}", labels.into_iter().format(" / ")))
                    .with_order(-(<Pos as Into<usize>>::into(start) as i32)),
            );
        }

        report.finish()
    }
}
