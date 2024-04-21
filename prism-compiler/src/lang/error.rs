use crate::lang::{TcEnv, UnionIndex, ValueOrigin};
use ariadne::{Color, Label, Report, ReportKind, Source};
use prism_parser::core::span::Span;
use std::io;

const SECONDARY_COLOR: Color = Color::RGB(0xA0, 0xA0, 0xA0);

#[derive(Debug)]
pub enum TypeError {
    ExpectType(UnionIndex),
    ExpectFn(UnionIndex),
    ExpectFnArg {
        function_type: UnionIndex,
        function_arg_type: UnionIndex,
        arg_type: UnionIndex,
    },
    IndexOutOfBound(UnionIndex),
    InfiniteType(UnionIndex, UnionIndex),
    BadInfer {
        free_var: UnionIndex,
        inferred_var: UnionIndex,
    },
}

impl TcEnv {
    pub fn report(&mut self, error: &TypeError) -> Option<Report<'static, Span>> {
        let report = Report::build(ReportKind::Error, (), 0);
        Some(match error {
            TypeError::ExpectType(i) => {
                let ValueOrigin::TypeOf(j) = self.value_origins[i.0] else {
                    unreachable!()
                };
                let ValueOrigin::SourceCode(span) = self.value_origins[j.0] else {
                    unreachable!()
                };
                let label = Label::new(span).with_message(format!(
                    "Expected a type, found value of type: {}",
                    self.index_to_sm_string(self.value_types[&j])
                ));

                report
                    .with_message("Expected type")
                    .with_label(label)
                    .finish()
            }
            TypeError::IndexOutOfBound(i) => {
                let ValueOrigin::SourceCode(span) = self.value_origins[i.0] else {
                    unreachable!()
                };
                let label = Label::new(span).with_message("This index is out of bounds.");

                report
                    .with_message("De Bruijn index out of bounds")
                    .with_label(label)
                    .finish()
            }
            TypeError::ExpectFn(i) => {
                let ValueOrigin::TypeOf(j) = self.value_origins[i.0] else {
                    unreachable!()
                };
                let ValueOrigin::SourceCode(span) = self.value_origins[j.0] else {
                    unreachable!()
                };
                let label = Label::new(span).with_message(format!(
                    "Expected a function, found value of type: {}",
                    self.index_to_sm_string(self.value_types[&j])
                ));
                report
                    .with_message("Expected function")
                    .with_label(label)
                    .finish()
            }
            TypeError::ExpectFnArg {
                function_type,
                function_arg_type,
                arg_type,
            } => {
                let ValueOrigin::TypeOf(j) = self.value_origins[arg_type.0] else {
                    unreachable!()
                };
                let ValueOrigin::SourceCode(span) = self.value_origins[j.0] else {
                    unreachable!()
                };
                let label_arg = Label::new(span).with_message(format!(
                    "This argument has type: {}",
                    self.index_to_sm_string(*arg_type)
                ));

                let ValueOrigin::TypeOf(j) = self.value_origins[function_type.0] else {
                    unreachable!()
                };
                let ValueOrigin::SourceCode(span) = self.value_origins[j.0] else {
                    unreachable!()
                };
                let label_fn = Label::new(span)
                    .with_message(format!(
                        "This function takes arguments of type: {}",
                        self.index_to_sm_string(*function_arg_type)
                    ))
                    .with_order(1)
                    .with_color(SECONDARY_COLOR);

                report
                    .with_message("Argument type mismatch in function application")
                    .with_label(label_arg)
                    .with_label(label_fn)
                    .finish()
            }
            TypeError::InfiniteType(left, right) => {
                let (left_span, left_description) = self.label_value(*left)?;
                let (right_span, right_description) = self.label_value(*left)?;

                report.with_message("Constraint creates an infinite type")
                    .with_label(Label::new(left_span).with_message(format!("Left side of constraint from {left_description}: {}", self.index_to_sm_string(*left))))
                    .with_label(Label::new(right_span).with_message(format!("Right side of constraint from {right_description}: {}", self.index_to_sm_string(*right))))
                    .with_help("If this doesn't obviously create an infinite type, I'm sorry. This is probably because of hidden constraints.")
                    .finish()
            },
            TypeError::BadInfer { .. } => report.finish(),
        })
    }

    fn label_value(&self, mut value: UnionIndex) -> Option<(Span, &'static str)> {
        let mut origin_description = "this value";
        let span = loop {
            match self.value_origins[value.0] {
                ValueOrigin::SourceCode(span) => break span,
                ValueOrigin::TypeOf(sub_value) => {
                    assert_eq!(origin_description, "this value");
                    origin_description = "type of this value";
                    value = sub_value;
                }
                ValueOrigin::FreeSub(v) => {
                    origin_description = "type of this value";
                    value = v
                },
                ValueOrigin::FreeValueFailure(_) => return None,
                ValueOrigin::FreeTypeFailure(_) => return None,
            }
        };
        Some((span, origin_description))
    }
}

pub struct AggregatedTypeError {
    pub errors: Vec<TypeError>,
}

impl AggregatedTypeError {
    pub fn eprint(&self, env: &mut TcEnv, input: &str) -> io::Result<()> {
        let mut input = Source::from(input);
        for report in self.errors.iter().flat_map(|err| env.report(err)) {
            report.eprint(&mut input)?;
        }
        Ok(())
    }
}

pub trait TypeResultExt<T> {
    fn unwrap_or_eprint(self, env: &mut TcEnv, input: &str) -> T;
}

impl<T> TypeResultExt<T> for Result<T, AggregatedTypeError> {
    fn unwrap_or_eprint(self, env: &mut TcEnv, input: &str) -> T {
        self.unwrap_or_else(|es| {
            es.eprint(env, input).unwrap();
            panic!("Failed to parse grammar")
        })
    }
}
