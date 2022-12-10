use crate::parser::core::error::ParseError;
use crate::parser::core::span::Span;
use itertools::Itertools;
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

    pub fn into_paths(self) -> Vec<Vec<L>> {
        let mut subs = self.1.into_iter().map(|t| t.into_paths()).concat();
        if let Some(l) = self.0 {
            if subs.is_empty() {
                subs.push(vec![l]);
            } else {
                subs.iter_mut().for_each(|v| {
                    v.push(l.clone());
                });
            }
        }
        subs
    }
}

#[derive(Clone, Debug)]
pub struct TreeError<L: Eq + Hash + Clone> {
    pub span: Span,
    pub labels: ErrorTree<L>,
}

impl<L: Eq + Hash + Clone> ParseError for TreeError<L> {
    type L = L;

    fn new(span: Span) -> Self {
        Self {
            span,
            labels: ErrorTree(None, vec![]),
        }
    }

    fn add_label(&mut self, label: L) {
        let mut tree = ErrorTree(None, vec![]);
        mem::swap(&mut self.labels, &mut tree);
        tree = tree.label(label);
        mem::swap(&mut self.labels, &mut tree);
    }

    fn merge(mut self, other: Self) -> Self {
        assert_eq!(self.span.start, other.span.start);
        self.labels = self.labels.merge(other.labels);
        Self {
            span: Span::new(self.span.start, max(self.span.end, other.span.end)),
            labels: self.labels,
        }
    }
}