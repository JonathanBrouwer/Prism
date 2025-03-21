use crate::core::input_table::InputTable;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use std::io;

pub struct AggregatedParseError<'grm, E: ParseError<L = ErrorLabel<'grm>> + 'grm> {
    pub input: &'grm InputTable<'grm>,
    pub errors: Vec<E>,
}

impl<'grm, E: ParseError<L = ErrorLabel<'grm>> + 'grm> AggregatedParseError<'grm, E> {
    pub fn eprint(&self) -> io::Result<()> {
        for e in &self.errors {
            e.report(false).eprint::<&InputTable<'grm>>(&self.input)?
        }
        Ok(())
    }
}

pub trait ParseResultExt<T> {
    fn unwrap_or_eprint(self) -> T;
}

impl<'grm, E: ParseError<L = ErrorLabel<'grm>> + 'grm, T> ParseResultExt<T>
    for Result<T, AggregatedParseError<'grm, E>>
{
    fn unwrap_or_eprint(self) -> T {
        self.unwrap_or_else(|es| {
            es.eprint().unwrap();
            panic!("Failed to parse")
        })
    }
}
