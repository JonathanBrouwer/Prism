use ariadne::Report;
use prism_parser::core::span::Span;
use crate::lang::{TcEnv, UnionIndex};

#[derive(Debug)]
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
        match error {
            TcError::ExpectEq(_, _) => todo!(),
            TcError::IndexOutOfBound(_) => todo!(),
            TcError::InfiniteType(_) => todo!(),
            TcError::BadInfer { .. } => todo!(),
        }

        // let base = Report::build(ReportKind::Error, (), span.start.into())


        
        todo!()
        
        
            //Header
            // .with_message("Parsing error")
            // .finish()
    }
}