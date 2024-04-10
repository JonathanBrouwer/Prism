use ariadne::{Config, LabelAttach, Report, ReportKind};
use prism_parser::core::span::Span;
use crate::coc::{TcEnv, UnionIndex};

pub enum TcError {
    ExpectEq(UnionIndex, UnionIndex),
    IndexOutOfBound(UnionIndex),
    InfiniteType(UnionIndex),
    BadInfer {
        free_var: UnionIndex,
        inferred_var: UnionIndex,
    }
}

impl TcEnv {
    pub fn error_to_report(&self, error: &TcError, input: &str) -> Report<'static, Span> {
        // let base = Report::build(ReportKind::Error, (), span.start.into())

        
        todo!()
        
        
            //Header
            // .with_message("Parsing error")
            // .finish()
    }
}