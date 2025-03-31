use crate::core::input_table::{InputTable, InputTableInner};
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use std::io;
use std::sync::Arc;

pub struct AggregatedParseError<E: ParseError<L = ErrorLabel>> {
    pub input: Arc<InputTable>,
    pub errors: Vec<E>,
}

impl<E: ParseError<L = ErrorLabel>> AggregatedParseError<E> {
    pub fn eprint(&self) -> io::Result<()> {
        for e in &self.errors {
            e.report(false, &self.input)
                .eprint::<&InputTableInner>(&*self.input.inner())?
        }
        Ok(())
    }
}

pub trait ParseResultExt<T> {
    fn unwrap_or_eprint(self) -> T;
}

impl<E: ParseError<L = ErrorLabel>, T> ParseResultExt<T> for Result<T, AggregatedParseError<E>> {
    fn unwrap_or_eprint(self) -> T {
        self.unwrap_or_else(|es| {
            es.eprint().unwrap();
            panic!("Failed to parse")
        })
    }
}
