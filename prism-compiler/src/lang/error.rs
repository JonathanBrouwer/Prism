use std::io;
use ariadne::{Color, Label, Report, ReportKind, Source};
use prism_parser::core::span::Span;
use crate::lang::{TcEnv, UnionIndex, ValueOrigin};

#[derive(Debug)]
pub enum TypeError {
    ExpectType(UnionIndex),
    IndexOutOfBound(UnionIndex),
    InfiniteType(UnionIndex),
    BadInfer {
        free_var: UnionIndex,
        inferred_var: UnionIndex,
    }
}

impl TcEnv {
    pub fn report(&mut self, error: &TypeError) -> Option<Report<'static, Span>> {
        let report = Report::build(ReportKind::Error, (), 0);
        Some(match error {
            TypeError::ExpectType(i) => {
                let label = match self.value_origins[i.0] {
                    ValueOrigin::SourceCode(_) => unreachable!(),
                    ValueOrigin::TypeOf(j) => {
                        match self.value_origins[j.0] {
                            ValueOrigin::SourceCode(span) => Label::new(span).with_message(format!("Expected a type, found value of type: {}", self.index_to_sm_string(self.value_types[&j]))),
                            _ => unreachable!(),
                        }
                    }
                    ValueOrigin::FreeSub(_) => unreachable!(),
                    ValueOrigin::FreeValueFailure(_) => unreachable!(),
                    ValueOrigin::FreeTypeFailure(_) => unreachable!(),
                    ValueOrigin::Test => unreachable!(),
                };
                report.with_message("Expected type")
                    .with_label(label)
                    .finish()
            },
            TypeError::IndexOutOfBound(_) => todo!(),
            TypeError::InfiniteType(_) => todo!(),
            TypeError::BadInfer { .. } => todo!(),
        })
    }
}

pub struct AggregatedTypeError {
    pub errors: Vec<TypeError>
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