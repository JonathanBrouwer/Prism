use crate::core::input_table::{InputTable, InputTableInner};
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use std::io;
use std::sync::Arc;

pub struct AggregatedParseError<'arn, E: ParseError<L = ErrorLabel<'arn>> + 'arn> {
    pub input: Arc<InputTable<'arn>>,
    pub errors: Vec<E>,
}

impl<'arn, E: ParseError<L = ErrorLabel<'arn>> + 'arn> AggregatedParseError<'arn, E> {
    pub fn eprint(&self) -> io::Result<()> {
        for e in &self.errors {
            e.report(false, &self.input)
                .eprint::<&InputTableInner<'arn>>(&*self.input.inner())?
        }
        Ok(())
    }
}

pub trait ParseResultExt<T> {
    fn unwrap_or_eprint(self) -> T;
}

impl<'arn, E: ParseError<L = ErrorLabel<'arn>> + 'arn, T> ParseResultExt<T>
    for Result<T, AggregatedParseError<'arn, E>>
{
    fn unwrap_or_eprint(self) -> T {
        self.unwrap_or_else(|es| {
            es.eprint().unwrap();
            panic!("Failed to parse")
        })
    }
}
