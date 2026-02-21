use crate::parser::ParserPrismEnv;
use prism_diag::{Annotation, AnnotationGroup, Diag};
use prism_input::pos::Pos;
use prism_input::span::Span;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub type PResult<T> = Result<T, ExpectedGuaranteed>;

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum Expected {
    Literal(String),
    Rule(String),
    EndOfFile,
}

impl Display for Expected {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expected::Literal(exp) => write!(f, "`{exp}`"),
            Expected::Rule(rule) => write!(f, "{rule}"),
            Expected::EndOfFile => write!(f, "end of file"),
        }
    }
}

pub struct ExpectedGuaranteed(());

pub struct ErrorState {
    fail_pos: Pos,
    expected: Vec<(Span, Expected)>,
}

impl ErrorState {
    pub fn new(pos: Pos) -> Self {
        Self {
            fail_pos: pos,
            expected: Vec::new(),
        }
    }
}

impl<'a> ParserPrismEnv<'a> {
    pub fn expect(&mut self, span: Span, expected: Expected) -> ExpectedGuaranteed {
        let fail_pos = span.start_pos();
        match fail_pos.cmp(&self.error_state.fail_pos) {
            Ordering::Less => {
                assert!(!self.error_state.expected.is_empty());
            }
            Ordering::Equal => {
                self.error_state.expected.push((span, expected));
            }
            Ordering::Greater => {
                self.error_state.fail_pos = fail_pos;
                self.error_state.expected.clear();
                self.error_state.expected.push((span, expected));
            }
        }
        ExpectedGuaranteed(())
    }

    pub fn expected_into_diag(&mut self) -> Option<Diag> {
        if self.error_state.expected.is_empty() {
            return None;
        }

        let mut labels_map: HashMap<Pos, (Pos, Vec<_>)> = HashMap::new();
        for (span, exp) in &self.error_state.expected {
            let (end_pos, expected) = labels_map
                .entry(span.start_pos())
                .or_insert((span.end_pos(), Vec::new()));
            *end_pos = (*end_pos).max(span.end_pos());
            expected.push(exp);
        }

        Some(Diag {
            title: "Parsing failed".into(),
            id: "Parser".into(),
            groups: vec![AnnotationGroup {
                annotations: labels_map
                    .into_iter()
                    .map(|(start, (end, mut labels))| Annotation {
                        span: start.span_to(end),
                        label: Some(match &labels[..] {
                            [] => unreachable!(),
                            [label] => format!("Expected: {}", label),
                            _ => {
                                labels.sort();
                                labels.dedup();

                                format!(
                                    "Expected one of: {}",
                                    labels
                                        .iter()
                                        .map(|v| v.to_string())
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                )
                            }
                        }),
                    })
                    .collect(),
            }],
        })
    }

    pub fn fail(&mut self) -> ExpectedGuaranteed {
        assert!(!self.error_state.expected.is_empty());
        ExpectedGuaranteed(())
    }
}
