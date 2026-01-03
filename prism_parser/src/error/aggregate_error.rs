use crate::error::ParseError;
use crate::error::error_label::ErrorLabel;
use prism_diag::RenderConfig;
use prism_input::input_table::InputTable;
use std::io;

#[must_use]
pub struct AggregatedParseError<E: ParseError<L = ErrorLabel>> {
    pub errors: Vec<E>,
}

impl<E: ParseError<L = ErrorLabel>> AggregatedParseError<E> {
    pub fn eprint(&self, input: &InputTable) -> io::Result<()> {
        for e in &self.errors {
            eprintln!(
                "{}\n",
                e.diag().render(&RenderConfig::default(), &input.inner())
            );
        }
        Ok(())
    }

    pub fn unwrap_or_eprint(&self, input: &InputTable) {
        if self.errors.is_empty() {
            return;
        }
        self.eprint(input).unwrap();
        panic!("Failed to parse")
    }
}
