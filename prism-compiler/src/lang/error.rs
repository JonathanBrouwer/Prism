use crate::lang::CoreIndex;
use crate::lang::env::DbEnv;
use crate::lang::{PrismDb, ValueOrigin};
use ariadne::{Color, Label, Report, ReportKind};
use prism_parser::core::span::Span;
use prism_parser::error::ParseError;
use prism_parser::error::set_error::SetError;
use std::mem;

const SECONDARY_COLOR: Color = Color::Rgb(0xA0, 0xA0, 0xA0);

pub enum PrismError {
    ParseError(SetError),
    TypeError(TypeError),
}

impl PrismError {
    pub fn eprint(&self, env: &mut PrismDb) {
        let report = match self {
            PrismError::ParseError(e) => e.report(),
            PrismError::TypeError(e) => env.report(e).unwrap(),
        };
        report.eprint(&*env.input.inner()).unwrap();
    }
}

impl PrismDb {
    pub fn eprint_errors(&mut self) {
        let errors = mem::take(&mut self.errors);
        for error in errors {
            error.eprint(self);
        }
    }

    pub fn assert_no_errors(&mut self) {
        if !self.errors.is_empty() {
            self.eprint_errors();
            panic!("Errors encounterd, see above");
        }
    }
}

pub enum TypeError {
    ExpectType(CoreIndex),
    ExpectFn(CoreIndex),
    ExpectFnArg {
        function_type: (CoreIndex, DbEnv),
        function_arg_type: (CoreIndex, DbEnv),
        arg_type: (CoreIndex, DbEnv),
    },
    ExpectTypeAssert {
        expr: CoreIndex,
        expr_type: CoreIndex,
        expected_type: CoreIndex,
    },
    IndexOutOfBound(CoreIndex),
    RecursionLimit(CoreIndex, CoreIndex),
    BadInfer {
        free_var: CoreIndex,
        inferred_var: CoreIndex,
    },
    UnknownName(Span),
}

impl PrismDb {
    pub fn report(&mut self, error: &TypeError) -> Option<Report<'static, Span>> {
        Some(match error {
            TypeError::ExpectType(i) => {
                let ValueOrigin::TypeOf(j) = self.checked_origins[**i] else {
                    unreachable!()
                };
                let ValueOrigin::SourceCode(span) = self.checked_origins[*j] else {
                    unreachable!()
                };

                Report::build(ReportKind::Error, span)
                    .with_message("Expected type")
                    .with_label(Label::new(span).with_message(format!(
                        "Expected a type, found value of type: {}",
                        self.index_to_sm_string(self.checked_types[&j])
                    )))
                    .finish()
            }
            TypeError::ExpectTypeAssert {
                expr,
                expr_type,
                expected_type,
            } => {
                let ValueOrigin::SourceCode(span_expr) = self.checked_origins[**expr] else {
                    unreachable!()
                };
                let ValueOrigin::SourceCode(span_expected) = self.checked_origins[**expected_type]
                else {
                    unreachable!()
                };

                Report::build(ReportKind::Error, span_expr)
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
                let ValueOrigin::SourceCode(span) = self.checked_origins[**i] else {
                    unreachable!()
                };

                Report::build(ReportKind::Error, span)
                    .with_message(format!("De Bruijn index `{}` out of bounds", *i))
                    .with_label(Label::new(span).with_message("This index is out of bounds."))
                    .finish()
            }
            TypeError::ExpectFn(i) => {
                let ValueOrigin::TypeOf(j) = self.checked_origins[**i] else {
                    unreachable!()
                };
                let ValueOrigin::SourceCode(span) = self.checked_origins[*j] else {
                    unreachable!()
                };
                Report::build(ReportKind::Error, span)
                    .with_message("Expected function")
                    .with_label(Label::new(span).with_message(format!(
                        "Expected a function, found value of type: {}",
                        self.index_to_sm_string(self.checked_types[&j])
                    )))
                    .finish()
            }
            TypeError::ExpectFnArg {
                function_type,
                function_arg_type,
                arg_type,
            } => {
                let span = self.source_span(arg_type.0);
                let label_arg = Label::new(span).with_message(format!(
                    "This argument has type: {}",
                    self.index_to_br_string(arg_type.0, &arg_type.1)
                ));

                let span = self.source_span(function_type.0);
                let label_fn = Label::new(span)
                    .with_message(format!(
                        "This function takes arguments of type: {}",
                        self.index_to_br_string(function_arg_type.0, &function_arg_type.1)
                    ))
                    .with_order(1)
                    .with_color(SECONDARY_COLOR);

                Report::build(ReportKind::Error, span)
                    .with_message("Argument type mismatch in function application")
                    .with_label(label_arg)
                    .with_label(label_fn)
                    .finish()
            }
            TypeError::BadInfer { .. } => panic!(),
            TypeError::UnknownName(name) => Report::build(ReportKind::Error, *name)
                .with_message("Undefined name within this scope.")
                .with_label(Label::new(*name).with_message("This name is undefined."))
                .finish(),
            TypeError::RecursionLimit(left, right) => {
                let (left_span, left_description) = self.label_value(*left)?;
                let (right_span, right_description) = self.label_value(*left)?;

                Report::build(ReportKind::Error, left_span)
                    .with_message("Constraint hit recursion limit")
                    .with_label(Label::new(left_span).with_message(format!(
                        "Left side of constraint from {left_description}: {}",
                        self.index_to_sm_string(*left)
                    )))
                    .with_label(Label::new(right_span).with_message(format!(
                        "Right side of constraint from {right_description}: {}",
                        self.index_to_sm_string(*right)
                    )))
                    .finish()
            }
        })
    }

    fn label_value(&self, mut value: CoreIndex) -> Option<(Span, &'static str)> {
        let mut origin_description = "this value";
        let span = loop {
            match self.checked_origins[*value] {
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

    fn source_span(&self, idx: CoreIndex) -> Span {
        let ValueOrigin::TypeOf(j) = self.checked_origins[idx.0] else {
            unreachable!()
        };
        let ValueOrigin::SourceCode(span) = self.checked_origins[*j] else {
            unreachable!()
        };
        span
    }
}
