use std::io;
use ariadne::{Color, Label, Report, ReportKind, Source};
use prism_parser::core::span::Span;
use crate::lang::{TcEnv, UnionIndex};

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
    pub fn report(&self, error: &TypeError) -> Report<'static, Span> {
        let report = Report::build(ReportKind::Error, (), 0);
        match error {
            TypeError::ExpectType(i) => {
                report.with_message("Expected type")
                    // .with_label(self.union_index_to_label(*i))
                    .finish()
            },
            TypeError::IndexOutOfBound(_) => todo!(),
            TypeError::InfiniteType(_) => todo!(),
            TypeError::BadInfer { .. } => todo!(),
        }
    }

    fn union_index_to_label(&self, i: UnionIndex) -> (Label<Span>, &'static str) {
        // Label::new(span)
        //     .with_message(match span.end - span.start {
        //         0 => "Failed to parse at this location (but recovered immediately)",
        //         1 => "This character was unparsable",
        //         _ => "These characters were unparsable",
        //     })
        //     .with_color(Color::Red)
        //     .with_priority(1)
        //     .with_order(i32::MIN),
        // )
        // match self.value_origins[i.0] {
        //     ValueOrigin::SourceCode(span) => (Label::new(span), "this value"),
        //     ValueOrigin::TypeOf(o) => (Label::new(span), "this value"),
        //     ValueOrigin::FreeSub(_) => todo!(),
        //     ValueOrigin::FreeValueFailure(_) => todo!(),
        //     ValueOrigin::FreeTypeFailure(_) => todo!(),
        //     ValueOrigin::Test => todo!(),
        // }
        todo!()
    }
}

pub struct AggregatedTypeError {
    pub errors: Vec<TypeError>
}

impl AggregatedTypeError {
    pub fn eprint(&self, env: &TcEnv, input: &str) -> io::Result<()> {
        for e in &self.errors {
            env.report(e).eprint(Source::from(input))?
        }
        Ok(())
    }
}

pub trait TypeResultExt<T> {
    fn unwrap_or_eprint(self, env: &TcEnv, input: &str) -> T;
}

impl<T> TypeResultExt<T> for Result<T, AggregatedTypeError> {
    fn unwrap_or_eprint(self, env: &TcEnv, input: &str) -> T {
        self.unwrap_or_else(|es| {
            es.eprint(env, input).unwrap();
            panic!("Failed to parse grammar")
        })
    }
}