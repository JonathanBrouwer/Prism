use crate::lang::UnionIndex;
use crate::lang::{TcEnv, ValueOrigin};
use ariadne::{Color, Label, Report, ReportKind, Source};
use prism_parser::core::pos::Pos;
use prism_parser::core::span::Span;
use std::io;

const SECONDARY_COLOR: Color = Color::Rgb(0xA0, 0xA0, 0xA0);

#[derive(Debug)]
pub enum TypeError {
    ExpectType(UnionIndex),
    ExpectFn(UnionIndex),
    ExpectFnArg {
        function_type: UnionIndex,
        function_arg_type: UnionIndex,
        arg_type: UnionIndex,
    },
    ExpectTypeAssert {
        expr: UnionIndex,
        expr_type: UnionIndex,
        expected_type: UnionIndex,
    },
    IndexOutOfBound(UnionIndex),
    InfiniteType(UnionIndex, UnionIndex),
    BadInfer {
        free_var: UnionIndex,
        inferred_var: UnionIndex,
    },
    UnknownName(Span),
}

impl TcEnv<'_> {
    pub fn report(&mut self, error: &TypeError) -> Option<Report<'static, Span>> {
        let report = Report::build(ReportKind::Error, Span::new(Pos::start(), Pos::start()));
        Some(match error {
            TypeError::ExpectType(i) => {
                let ValueOrigin::TypeOf(j) = self.value_origins[**i] else {
                    unreachable!()
                };
                let ValueOrigin::SourceCode(span) = self.value_origins[*j] else {
                    unreachable!()
                };

                report
                    .with_message("Expected type")
                    .with_label(Label::new(span).with_message(format!(
                        "Expected a type, found value of type: {}",
                        self.index_to_sm_string(self.value_types[&j])
                    )))
                    .finish()
            }
            TypeError::ExpectTypeAssert {
                expr,
                expr_type,
                expected_type,
            } => {
                let ValueOrigin::SourceCode(span_expr) = self.value_origins[**expr] else {
                    unreachable!()
                };
                let ValueOrigin::SourceCode(span_expected) = self.value_origins[**expected_type]
                else {
                    unreachable!()
                };

                report
                    .with_message("Type assertion failed")
                    .with_label(Label::new(span_expr).with_message(format!(
                        "This value has type: {}",
                        self.index_to_sm_string(*expr_type)
                    )))
                    .with_label(Label::new(span_expected).with_message(format!(
                        "Expected value to have this type: {}",
                        self.index_to_sm_string(*expected_type)
                    )))
                    .finish()
            }
            TypeError::IndexOutOfBound(i) => {
                let ValueOrigin::SourceCode(span) = self.value_origins[**i] else {
                    unreachable!()
                };

                report
                    .with_message(format!("De Bruijn index `{}` out of bounds", *i))
                    .with_label(Label::new(span).with_message("This index is out of bounds."))
                    .finish()
            }
            TypeError::ExpectFn(i) => {
                let ValueOrigin::TypeOf(j) = self.value_origins[**i] else {
                    unreachable!()
                };
                let ValueOrigin::SourceCode(span) = self.value_origins[*j] else {
                    unreachable!()
                };
                report
                    .with_message("Expected function")
                    .with_label(Label::new(span).with_message(format!(
                        "Expected a function, found value of type: {}",
                        self.index_to_sm_string(self.value_types[&j])
                    )))
                    .finish()
            }
            TypeError::ExpectFnArg {
                function_type,
                function_arg_type,
                arg_type,
            } => {
                let ValueOrigin::TypeOf(j) = self.value_origins[**arg_type] else {
                    unreachable!()
                };
                let ValueOrigin::SourceCode(span) = self.value_origins[*j] else {
                    unreachable!()
                };
                let label_arg = Label::new(span).with_message(format!(
                    "This argument has type: {}",
                    self.index_to_sm_string(*arg_type)
                ));

                let ValueOrigin::TypeOf(j) = self.value_origins[**function_type] else {
                    unreachable!()
                };
                let ValueOrigin::SourceCode(span) = self.value_origins[*j] else {
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

                let constraints = self
                    .queued_beq_free
                    .iter()
                    .flat_map(|(i, cs)| cs.iter().map(move |c| format!("{:?} = {:?}", i, c.1 .0)))
                    .collect::<Vec<_>>()
                    .join(",");

                report.with_message("Constraint creates an infinite type")
                    .with_label(Label::new(left_span).with_message(format!("Left side of constraint from {left_description}: {}", self.index_to_sm_string(*left))))
                    .with_label(Label::new(right_span).with_message(format!("Right side of constraint from {right_description}: {}", self.index_to_sm_string(*right))))
                    .with_help(format!("If this doesn't obviously create an infinite type, I'm sorry. This is probably because of the following hidden constraints: [{constraints}]"))
                    .finish()
            }
            TypeError::BadInfer { .. } => report.finish(),
            TypeError::UnknownName(name) => report
                .with_message("Undefined name within this scope.")
                .with_label(Label::new(*name).with_message("This name is undefined."))
                .finish(),
        })
    }

    fn label_value(&self, mut value: UnionIndex) -> Option<(Span, &'static str)> {
        let mut origin_description = "this value";
        let span = loop {
            match self.value_origins[*value] {
                ValueOrigin::SourceCode(span) => break span,
                ValueOrigin::TypeOf(sub_value) => {
                    assert_eq!(origin_description, "this value");
                    origin_description = "type of this value";
                    value = sub_value;
                }
                ValueOrigin::FreeSub(v) => value = v,
                ValueOrigin::Failure => return None,
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
            panic!("Failed to type check")
        })
    }
}
