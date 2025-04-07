use crate::core::input_table::{InputTable, InputTableInner};
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use std::io;
use std::sync::Arc;

#[must_use]
pub struct AggregatedParseError<E: ParseError<L = ErrorLabel>> {
    pub errors: Vec<E>,
}

impl<E: ParseError<L = ErrorLabel>> AggregatedParseError<E> {
    pub fn eprint(&self, input: &InputTable) -> io::Result<()> {
        for e in &self.errors {
            e.report().eprint::<&InputTableInner>(&*input.inner())?
        }
        Ok(())
    }

    pub fn unwrap_or_eprint(&self, input: &InputTable) {
        if self.errors.len() == 0 {
            return;
        }
        self.eprint(input).unwrap();
        panic!("Failed to parse")
    }
}
