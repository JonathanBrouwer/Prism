use crate::error::ParseError;
use crate::error::error_label::ErrorLabel;
use prism_diags::Diag;
use prism_input::pos::Pos;
use prism_input::span::Span;
use std::cmp::max;
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
pub struct TreeError {
    pub span: Span,
    pub labels: ErrorTree<ErrorLabel>,
}

impl TreeError {
    fn add_label(&mut self, label: ErrorLabel) {
        let mut tree = ErrorTree(None, vec![]);
        mem::swap(&mut self.labels, &mut tree);
        tree = tree.label(label);
        mem::swap(&mut self.labels, &mut tree);
    }
}

impl ParseError for TreeError {
    type L = ErrorLabel;

    fn new(pos: Pos) -> Self {
        Self {
            span: Span::new(pos, 0),
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
        assert_eq!(self.span.start_pos(), other.span.start_pos());
        self.labels = self.labels.merge(other.labels);
        Self {
            span: Span::new(
                self.span.start_pos(),
                max(self.span.len(), other.span.len()),
            ),
            labels: self.labels,
        }
    }

    fn span(&self) -> Span {
        self.span
    }

    fn set_end(&mut self, end: Pos) {
        self.span = Span::new_with_end(self.span.start_pos(), end);
    }

    fn diag(&self) -> Diag {
        todo!()
        // let mut report: ReportBuilder<Span> = base_report(self.span);
        //
        // //Add labels
        // for path in self.labels.into_paths() {
        //     if path.is_empty() {
        //         continue;
        //     }
        //     let label = &path[0];
        //
        //     report = report.with_label(
        //         Label::new(label.span())
        //             .with_message(
        //                 path.iter()
        //                     .map(|v| v.to_string())
        //                     .collect::<Vec<_>>()
        //                     .join(" <- ")
        //                     .to_string(),
        //             )
        //             .with_order(-(label.span().start_pos().idx_in_file() as i32)),
        //     );
        // }
        //
        // report.finish()
    }
}
