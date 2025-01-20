use crate::core::pos::Pos;
use crate::core::span::Span;
use crate::error::error_printer::{base_report, ErrorLabel};
use crate::error::ParseError;
use ariadne::{Label, Report, ReportBuilder};
use std::hash::Hash;
use std::mem;

#[derive(Clone, Debug)]
pub struct ErrorTree<L: Eq + Hash + Clone>(Option<L>, Vec<Self>);

impl<L: Eq + Hash + Clone> ErrorTree<L> {
    fn merge(mut self, mut other: Self) -> Self {
        if self.0.is_none() && other.0.is_none() {
            self.1.append(&mut other.1);
            self
        } else {
            ErrorTree(None, vec![self, other])
        }
    }

    fn label(self, l: L) -> Self {
        if self.0.is_none() {
            ErrorTree(Some(l), self.1)
        } else {
            ErrorTree(Some(l), vec![self])
        }
    }

    pub fn into_paths(&self) -> Vec<Vec<&L>> {
        let mut subs = self
            .1
            .iter()
            .flat_map(|t| t.into_paths())
            .collect::<Vec<_>>();
        if let Some(l) = &self.0 {
            if subs.is_empty() {
                subs.push(vec![l]);
            } else {
                subs.iter_mut().for_each(|v| {
                    v.push(l);
                });
            }
        }
        subs
    }
}

/// ErrorTree keeps track of all information that it is provided, it is really verbose
#[derive(Clone)]
pub struct TreeError<'grm> {
    pub pos: Pos,
    pub labels: ErrorTree<ErrorLabel<'grm>>,
}

impl<'grm> TreeError<'grm> {
    fn add_label(&mut self, label: ErrorLabel<'grm>) {
        let mut tree = ErrorTree(None, vec![]);
        mem::swap(&mut self.labels, &mut tree);
        tree = tree.label(label);
        mem::swap(&mut self.labels, &mut tree);
    }
}

impl<'grm> ParseError for TreeError<'grm> {
    type L = ErrorLabel<'grm>;

    fn new(pos: Pos) -> Self {
        Self {
            pos,
            labels: ErrorTree(None, vec![]),
        }
    }

    fn add_label_explicit(&mut self, l: Self::L) {
        self.add_label(l)
    }

    fn add_label_implicit(&mut self, l: Self::L) {
        self.add_label(l)
    }

    fn merge(mut self, other: Self) -> Self {
        assert_eq!(self.pos, other.pos);
        self.labels = self.labels.merge(other.labels);
        Self {
            pos: self.pos,
            labels: self.labels,
        }
    }

    fn report(&self, enable_debug: bool) -> Report<'static, Span> {
        let mut report: ReportBuilder<Span> = base_report(self.pos.span_to(self.pos));

        //Add labels
        for path in self.labels.into_paths() {
            let path = path
                .iter()
                .filter(|l| enable_debug || !l.is_debug())
                .collect::<Vec<_>>();
            if path.is_empty() {
                continue;
            }
            let label = &path[0];

            report = report.with_label(
                Label::new(label.span())
                    .with_message(
                        path.iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(" <- ")
                            .to_string(),
                    )
                    .with_order(-(<Pos as Into<usize>>::into(label.span().start) as i32)),
            );
        }

        report.finish()
    }
}
